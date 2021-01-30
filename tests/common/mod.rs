use diesel::prelude::*;
use diesel_migrations;
use harbour_whisperfish::store::temp;
use harbour_whisperfish::store::{NewMessage, NewSession};
use harbour_whisperfish::store::{Storage, StorageLocation};
use rstest::fixture;

pub type InMemoryDb = (Storage, StorageLocation<tempdir::TempDir>);

/// We do not want to test on a live db, use temporary dir
#[fixture]
pub fn in_memory_db() -> InMemoryDb {
    || -> Result<InMemoryDb, failure::Error> {
        let temp = temp();
        std::fs::create_dir(temp.join("db"))?;
        std::fs::create_dir(temp.join("storage"))?;
        Ok((Storage::open_database_only(&temp)?, temp))
    }()
    .expect("initialized storage")
}

/// Setup helper for basic, empty database
pub fn setup_db(in_memory_db: &Storage) {
    let db = in_memory_db.db.lock();
    let conn = db.unwrap();

    diesel_migrations::run_pending_migrations(&*conn).unwrap()
}

/// Setup helper for creating a session
pub fn setup_session(in_memory_db: &Storage, new_session: &NewSession) -> usize {
    use harbour_whisperfish::schema::session::dsl::*;

    let db = in_memory_db.db.lock();
    let conn = db.unwrap();

    let query = diesel::insert_into(session).values(new_session);

    let res = match query.execute(&*conn) {
        Ok(rows_inserted) => rows_inserted,
        Err(error) => panic!(error.to_string()),
    };

    res
}

/// Setup helper for creating a proper chat
/// where each message in `Vec<NewMessage>`
/// would be received by the message processor
pub fn setup_messages(in_memory_db: &Storage, new_messages: Vec<NewMessage>) -> usize {
    use harbour_whisperfish::schema::message::dsl::*;

    let db = in_memory_db.db.lock();
    let conn = db.unwrap();

    let query = diesel::insert_into(message).values(new_messages);

    query.execute(&*conn).expect("failed")
}
