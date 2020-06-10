use rstest::rstest;

use harbour_whisperfish::model::MessageModel;
use harbour_whisperfish::store::Storage;
use harbour_whisperfish::store::{NewMessage, NewSession};
use qmetaobject::QString;

mod common;
use common::*;

#[rstest]#[actix_rt::test]
async fn test_load_and_row_count(in_memory_db: Storage) {
    let session_config = NewSession {
        source: String::from("+358501234567"),
        message: String::from("whisperfish on paras:DDDD ja signal:DDD"),
        timestamp: 0,
        sent: true,
        received: false,
        unread: false,
        is_group: false,
        group_id: None,
        group_name: None,
        group_members: None,
        has_attachment: false,
    };

    setup_db(&in_memory_db);
    setup_session(&in_memory_db, &session_config);

    let new_messages = vec![NewMessage {
        session_id: 1,
        source: String::from("+358501234567"),
        text: String::from("nyt joni ne velat!"),
        timestamp: 1024,
        sent: false,
        received: true,
        flags: 0,
        attachment: None,
        mime_type: None,
        has_attachment: false,
        outgoing: false,
    }];

    setup_messages(&in_memory_db, new_messages);

    // Actual testing
    let mut mm = MessageModel::default();

    assert_eq!(mm.row_count(), 0);
    mm.load(1, QString::from("WHY IS THIS NEEDED WHEN IT ISN'T NEEDED ELSEWHERE?!"));
    assert_eq!(mm.row_count(), 1);
}
