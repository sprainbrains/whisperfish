#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::io::Read;
use std::path::Path;

use diesel::dsl::*;
use diesel::prelude::*;
use harbour_whisperfish::store;

#[path = "../../tests/migrations/schemas/mod.rs"]
pub mod schemas;

embed_migrations!();

fn derive_db_key(password: &str, salt_path: &Path) -> Result<[u8; 32], failure::Error> {
    let mut salt_file = std::fs::File::open(salt_path)?;
    let mut salt = [0u8; 8];
    failure::ensure!(salt_file.read(&mut salt)? == 8, "salt file not 8 bytes");

    let params = scrypt::Params::new(14, 8, 1)?;
    let mut key = [0u8; 32];
    scrypt::scrypt(password.as_bytes(), &salt, &params, &mut key)?;
    log::trace!("Computed the key, salt was {:?}", salt);
    Ok(key)
}

// XXX print message counts separate for sent and received, split up in groups and dm.
fn print_original_stats(db: &SqliteConnection) -> Result<(), failure::Error> {
    use schemas::original as schema;

    {
        use schema::session::dsl::*;
        let session_count: i64 = session.select(count(id)).first(db)?;
        let group_session_count: i64 = session.select(count(id)).filter(is_group).first(db)?;
        let non_group_session_count: i64 = session
            .select(count(id))
            .filter(is_group.eq(false))
            .first(db)?;
        println!("Session count: {}", session_count);
        println!("├ of which groups: {}", group_session_count);
        println!("└ of which direct sessions: {}", non_group_session_count);
    }

    {
        use schema::message::dsl::*;
        let message_count: i64 = message.select(count(id)).first(db)?;

        let session_on_id = schema::session::table.on(schema::session::id.eq(session_id));

        let non_group_message_count: i64 = message
            .left_join(session_on_id)
            .select(count(id))
            .filter(schema::session::is_group.eq(false))
            .first(db)?;
        let non_group_sent_count: i64 = message
            .left_join(session_on_id)
            .select(count(id))
            .filter(schema::session::is_group.eq(false))
            .filter(outgoing)
            .first(db)?;
        let non_group_received_count: i64 = message
            .left_join(session_on_id)
            .select(count(id))
            .filter(schema::session::is_group.eq(false))
            .filter(outgoing.eq(false))
            .first(db)?;

        let group_message_count: i64 = message
            .left_join(session_on_id)
            .select(count(id))
            .filter(schema::session::is_group)
            .first(db)?;
        let group_sent_count: i64 = message
            .left_join(session_on_id)
            .select(count(id))
            .filter(schema::session::is_group)
            .filter(outgoing)
            .first(db)?;
        let group_received_count: i64 = message
            .left_join(session_on_id)
            .select(count(id))
            .filter(schema::session::is_group)
            .filter(outgoing.eq(false))
            .first(db)?;

        let attachment_count: i64 = message.select(count(id)).filter(has_attachment).first(db)?;
        println!("Message count: {}", message_count);
        println!("├ with an attachment: {}", attachment_count);
        println!("├ of which group messages: {}", group_message_count);
        println!("│ ├ of which you sent: {}", group_sent_count);
        println!("│ └ of which you received: {}", group_received_count);
        println!("└ of which direct messages: {}", non_group_message_count);
        println!("  ├ of which you sent: {}", non_group_sent_count);
        println!("  └ of which you received: {}", non_group_received_count);
    }
    Ok(())
}

fn print_current_stats(db: &SqliteConnection) -> Result<(), failure::Error> {
    use schemas::current as schema;

    {
        use schema::recipients::dsl::*;
        let recipient_count: i64 = recipients.select(count(id)).first(db)?;
        println!("Recipients count: {}", recipient_count);
        let e164_count: i64 = recipients
            .select(count(id))
            .filter(e164.is_not_null())
            .first(db)?;
        let uuid_count: i64 = recipients
            .select(count(id))
            .filter(uuid.is_not_null())
            .first(db)?;
        println!("├ of which have a phone number: {}", e164_count);
        println!("└ of which have a uuid: {}", uuid_count);
    }
    {
        use schema::sessions::dsl::*;
        let session_count: i64 = sessions.select(count(id)).first(db)?;
        let group_v1_session_count: i64 = sessions
            .select(count(id))
            .filter(group_v1_id.is_not_null())
            .first(db)?;
        let dm_session_count: i64 = sessions
            .select(count(id))
            .filter(direct_message_recipient_id.is_not_null())
            .first(db)?;
        println!("Session count: {}", session_count);
        println!("├ of which group v1: {}", group_v1_session_count);
        println!("└ of which direct sessions: {}", dm_session_count);
    }

    {
        use schema::messages::dsl::*;
        let message_count: i64 = messages.select(count(id)).first(db)?;
        let group_v1_message_count: i64 = messages
            .left_join(schema::sessions::table)
            .select(count(id))
            .filter(schema::sessions::group_v1_id.is_not_null())
            .first(db)?;
        let group_v1_sent_count: i64 = messages
            .left_join(schema::sessions::table)
            .select(count(id))
            .filter(schema::sessions::group_v1_id.is_not_null())
            .filter(is_outbound)
            .first(db)?;
        let group_v1_received_count: i64 = messages
            .left_join(schema::sessions::table)
            .select(count(id))
            .filter(schema::sessions::group_v1_id.is_not_null())
            .filter(is_outbound.eq(false))
            .first(db)?;

        let direct_message_count: i64 = messages
            .left_join(schema::sessions::table)
            .select(count(id))
            .filter(schema::sessions::direct_message_recipient_id.is_not_null())
            .first(db)?;
        let direct_sent_count: i64 = messages
            .left_join(schema::sessions::table)
            .select(count(id))
            .filter(schema::sessions::direct_message_recipient_id.is_not_null())
            .filter(is_outbound)
            .first(db)?;
        let direct_received_count: i64 = messages
            .left_join(schema::sessions::table)
            .select(count(id))
            .filter(schema::sessions::direct_message_recipient_id.is_not_null())
            .filter(is_outbound.eq(false))
            .first(db)?;

        println!("Message count: {}", message_count);
        println!("├ of which group v1 messages: {}", group_v1_message_count);
        println!("│ ├ of which you sent: {}", group_v1_sent_count);
        println!("│ └ of which you received: {}", group_v1_received_count);
        println!("├ of which direct messages: {}", direct_message_count);
        println!("│ ├ of which you sent: {}", direct_sent_count);
        println!("│ └ of which you received: {}", direct_received_count);
    }
    {
        use schema::attachments::dsl::*;
        let attachment_count: i64 = attachments.select(count(id)).first(db)?;
        println!("└ total attachments: {}", attachment_count);
    }
    Ok(())
}

fn main() -> Result<(), failure::Error> {
    println!("This utility will test whether the Whisperfish database will successfully get migrated to the most recent format.");
    println!("It is a *dry-run*, which practically means we will work on a copy of the original database.");

    let storage = store::default_location().unwrap();
    let original_db_location = storage.join("db").join("harbour-whisperfish.db");
    println!("Location of the database: {:?}", original_db_location);

    println!("We now ask you your Whisperfish password.");
    let password =
        rpassword::read_password_from_tty(Some("Whisperfish storage password: ")).unwrap();

    // Copy the database to /tmp
    let db_location = Path::new("/tmp/wf-migration-dry-run.db");
    std::fs::copy(&original_db_location, db_location)?;
    // Don't keep any pointer to the original db, just in case I'm a crappy programmer.
    drop(original_db_location);
    println!("Location of the copied database: {:?}", db_location);

    let db = SqliteConnection::establish(db_location.to_str().unwrap())?;
    println!("The copy of the database has been opened.");
    let db_salt_path = storage.join("db").join("salt");
    let db_key = derive_db_key(&password, &db_salt_path)?;
    println!("Derived the db key.");

    db.execute(&format!("PRAGMA key = \"x'{}'\";", hex::encode(db_key)))?;
    db.execute("PRAGMA cipher_page_size = 4096;")?;

    println!("The copy of the database has been decrypted.");

    println!("------");
    if let Err(e) = print_original_stats(&db) {
        println!("Could not print \"original schema\" statistics: {}", e);
    }

    println!("------");
    let start = std::time::Instant::now();
    embedded_migrations::run(&db).unwrap();
    let end = std::time::Instant::now();
    println!("Migrations took {:?}", end - start);
    println!("------");

    print_current_stats(&db)?;
    println!("------");
    println!("Here above, the dry run should have produced at least two sets of statistics of your data.");
    println!("These should give a decent indication to whether some data has been lost. Please report a bug if so.");

    Ok(())
}
