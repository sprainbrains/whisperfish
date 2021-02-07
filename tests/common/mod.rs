use diesel::prelude::*;
use harbour_whisperfish::store::temp;
use harbour_whisperfish::store::{NewMessage, NewSession};
use harbour_whisperfish::store::{Storage, StorageLocation};

pub type InMemoryDb = (Storage, StorageLocation<tempdir::TempDir>);

/// We do not want to test on a live db, use temporary dir
pub async fn get_in_memory_db() -> InMemoryDb {
    let temp = temp();
    (
        Storage::new(&temp, None, 12345, "Some Password", [0; 52])
            .await
            .expect("Failed to initalize storage"),
        temp,
    )
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
