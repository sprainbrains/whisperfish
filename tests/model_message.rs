use rstest::{fixture,rstest};

use diesel::prelude::*;
use diesel_migrations;

use harbour_whisperfish::store::NewSession;
use harbour_whisperfish::store::Storage;
use harbour_whisperfish::store::memory;

/// We do not want to test on a live db, use :memory:
#[fixture]
fn in_memory_db() -> Storage {
    Storage::open(&memory()).unwrap()
}

/// Setup helper for basic, empty database
fn setup_db(in_memory_db: &Storage) {
    let db = in_memory_db.db.lock();
    let conn = db.unwrap();

    diesel_migrations::run_pending_migrations(&*conn).unwrap()
}

/// Setup helper for creating a session
fn setup_session(in_memory_db: &Storage, new_session: &NewSession) -> usize {
    use harbour_whisperfish::schema::session::dsl::*;

    let db = in_memory_db.db.lock();
    let conn = db.unwrap();

    let query = diesel::insert_into(session).values(new_session);

    let res = match query.execute(&*conn) {
        Ok(rows_inserted) => rows_inserted,
        Err(error) => panic!(error.to_string()),
    };

    return res;
}

#[rstest]
fn test_fetch_session_none(in_memory_db: Storage) {
    setup_db(&in_memory_db);

    let session = in_memory_db.fetch_session(1);
    assert!(session.is_none());
}

#[rstest]
fn test_fetch_group_session(in_memory_db: Storage) {
    let session_config = NewSession {
        source: String::from("+358501234567"),
        message: String::from("whisperfish on paras:DDDD ja signal:DDD"),
        timestamp: 0,
        sent: true,
        received: false,
        unread: false,
        is_group: true,
        group_id: None,
        group_name: Some(String::from("Spurdospärde")),
        group_members: Some(String::from("Joni,Viljami,Make,Spurdoliina")),
        has_attachment: false,
    };

    setup_db(&in_memory_db);
    setup_session(&in_memory_db, &session_config);

    let res = in_memory_db.fetch_session(1);
    assert!(res.is_some());

    let session = res.unwrap();
    assert_eq!(session.id, 1);
    assert_eq!(session.source, String::from("+358501234567"));
    assert_eq!(session.group_name, Some(String::from("Spurdospärde")));
    assert_eq!(session.message, String::from("whisperfish on paras:DDDD ja signal:DDD"));
}

#[rstest]
fn test_fetch_session_with_other(in_memory_db: Storage) {
    let session_config_1 = NewSession {
        source: String::from("foo"),
        message: String::from("first"),
        timestamp: 0,
        sent: true,
        received: false,
        unread: false,
        is_group: false,
        group_id: None,
        group_name: Some(String::from("")),
        group_members: Some(String::from("")),
        has_attachment: false,
    };

    let session_config_2 = NewSession {
        source: String::from("31337"),
        message: String::from("31337"),
        timestamp: 0,
        sent: false,
        received: true,
        unread: false,
        is_group: false,
        group_id: None,
        group_name: Some(String::from("")),
        group_members: Some(String::from("")),
        has_attachment: false,
    };

    setup_db(&in_memory_db);
    setup_session(&in_memory_db, &session_config_1);
    setup_session(&in_memory_db, &session_config_2);

    // Test retrieving the sessions in reverse order

    let res = in_memory_db.fetch_session(2);
    assert!(res.is_some());

    let session = res.unwrap();
    assert_eq!(session.id, 2);
    assert_eq!(session.source, String::from("31337"));
    assert_eq!(session.message, String::from("31337"));
    assert_eq!(session.group_name, Some(String::from("")));

    let res = in_memory_db.fetch_session(1);
    assert!(res.is_some());

    let session = res.unwrap();
    assert_eq!(session.id, 1);
    assert_eq!(session.source, String::from("foo"));
    assert_eq!(session.message, String::from("first"));
    assert_eq!(session.group_name, Some(String::from("")));
}

#[rstest]
fn test_fetch_all_messages_none(in_memory_db: Storage) {
    setup_db(&in_memory_db);

    let messages = in_memory_db.fetch_all_messages(1).unwrap();
    assert_eq!(messages.len(), 0);
}
