mod common;

use self::common::*;
use chrono::prelude::*;
use libsignal_service::content::Reaction;
use libsignal_service::proto::DataMessage;
use phonenumber::PhoneNumber;
use rstest::rstest;
use std::future::Future;
use std::sync::Arc;
use whisperfish::config::SignalConfig;
use whisperfish::store;
use whisperfish::store::orm::UnidentifiedAccessMode;
use whisperfish::store::{GroupV1, NewMessage, Storage};

#[rstest]
#[actix_rt::test]
async fn fetch_session_none(storage: impl Future<Output = InMemoryDb>) {
    let (storage, _temp_dir) = storage.await;

    let session = storage.fetch_session_by_id(1);
    assert!(session.is_none());
}

#[rstest]
#[actix_rt::test]
async fn insert_and_fetch_dm(storage: impl Future<Output = InMemoryDb>) {
    let (storage, _temp_dir) = storage.await;

    let phonenumber = phonenumber::parse(None, "+358501234567").unwrap();

    let inserted = storage.fetch_or_insert_session_by_phonenumber(&phonenumber);
    assert_eq!(inserted.id, 1);

    let session = storage.fetch_session_by_id(inserted.id).unwrap();
    let recipient = session.unwrap_dm();

    assert_eq!(session.id, inserted.id);
    assert_eq!(recipient.e164, Some(phonenumber));
}

#[rstest]
#[actix_rt::test]
async fn insert_and_fetch_group_session(storage: impl Future<Output = InMemoryDb>) {
    let (storage, _temp_dir) = storage.await;

    let group_id_hex = "1213dc10";
    let group_id = hex::decode(group_id_hex).unwrap();

    let group = GroupV1 {
        id: group_id,
        name: "Spurdospärde".into(),
        members: vec![
            phonenumber::parse(None, "+32474000000").unwrap(),
            phonenumber::parse(None, "+32475000000").unwrap(),
        ],
    };

    let inserted = storage.fetch_or_insert_session_by_group_v1(&group);

    let session = storage.fetch_session_by_id(inserted.id).unwrap();
    let fetched_group = session.unwrap_group_v1();

    assert_eq!(session.id, 1);
    assert_eq!(fetched_group.id, group_id_hex);
    assert_eq!(fetched_group.name, group.name);

    let mut members = storage.fetch_group_members_by_group_v1_id(&fetched_group.id);
    members.sort_by_key(|(_member, recipient)| recipient.e164.as_ref().map(PhoneNumber::to_string));

    assert_eq!(members.len(), group.members.len());
    assert_eq!(members[0].1.e164.as_ref(), Some(&group.members[0]));
    assert_eq!(members[1].1.e164.as_ref(), Some(&group.members[1]));
}

#[rstest]
#[actix_rt::test]
async fn fetch_two_distinct_session(storage: impl Future<Output = InMemoryDb>) {
    let (storage, _temp_dir) = storage.await;

    let pn1 = phonenumber::parse(None, "+32474000000").unwrap();
    let pn2 = phonenumber::parse(None, "+32475000000").unwrap();

    let session_1_inserted = storage.fetch_or_insert_session_by_phonenumber(&pn1);
    let session_2_inserted = storage.fetch_or_insert_session_by_phonenumber(&pn2);

    assert_ne!(session_1_inserted.id, session_2_inserted.id);

    // Test retrieving the sessions in reverse order
    let session = storage.fetch_session_by_id(session_2_inserted.id).unwrap();
    let recipient = session.unwrap_dm();
    assert_eq!(session.id, 2);
    assert_eq!(recipient.e164.as_ref(), Some(&pn2));

    let session = storage.fetch_session_by_id(session_1_inserted.id).unwrap();
    let recipient = session.unwrap_dm();
    assert_eq!(session.id, 1);
    assert_eq!(recipient.e164.as_ref(), Some(&pn1));
}

#[rstest]
#[actix_rt::test]
async fn fetch_messages_without_session(storage: impl Future<Output = InMemoryDb>) {
    let (storage, _temp_dir) = storage.await;

    let messages = storage.fetch_all_messages(1);
    assert_eq!(messages.len(), 0);
}

#[rstest]
#[actix_rt::test]
async fn process_message_no_session_source(storage: impl Future<Output = InMemoryDb>) {
    let (storage, _temp_dir) = storage.await;

    // First assert that session 1 does not exist.
    let session = storage.fetch_session_by_id(1);
    assert!(session.is_none());

    let pn1 = phonenumber::parse(None, "+32474000000").unwrap();

    // Now try to add a message.
    let new_message = NewMessage {
        session_id: Some(1),
        source_e164: Some(pn1),
        source_uuid: None,
        text: String::from("MSG 1"),
        timestamp: Utc::now().naive_utc(),
        sent: false,
        received: true,
        is_read: false,
        flags: 0,
        attachment: None,
        mime_type: None,
        has_attachment: false,
        outgoing: false,
        is_unidentified: false,
        quote_timestamp: None,
    };

    let (msg_inserted, session_inserted) = storage.process_message(new_message.clone(), None);

    // Test a session was created
    let session_fetch = storage
        .fetch_session_by_id(session_inserted.id)
        .expect("session has been created");
    assert_eq!(session_fetch.id, session_inserted.id);

    assert_eq!(
        msg_inserted.text.as_deref(),
        Some(&new_message.text as &str)
    );
}

#[rstest]
#[actix_rt::test]
async fn process_message_unresolved_session_source_resolved(
    storage: impl Future<Output = InMemoryDb>,
) {
    let (storage, _temp_dir) = storage.await;

    let pn1 = phonenumber::parse(None, "+32474000000").unwrap();

    let new_message = NewMessage {
        session_id: None,
        source_e164: Some(pn1),
        source_uuid: None,
        text: String::from("MSG 1"),
        timestamp: Utc::now().naive_utc(),
        sent: false,
        received: true,
        is_read: true,
        flags: 0,
        attachment: None,
        mime_type: None,
        has_attachment: false,
        outgoing: false,
        is_unidentified: false,
        quote_timestamp: None,
    };

    let (_msg, session) = storage.process_message(new_message, None);

    let messages = storage.fetch_all_messages(session.id);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].session_id, session.id);
}

#[rstest]
#[actix_rt::test]
async fn process_message_exists_session_source(storage: impl Future<Output = InMemoryDb>) {
    let (storage, _temp_dir) = storage.await;

    let pn1 = phonenumber::parse(None, "+358501234567").unwrap();
    let sess1 = storage.fetch_or_insert_session_by_phonenumber(&pn1);

    for second in 1..11 {
        let timestamp = Utc.timestamp_opt(second, 0).unwrap().naive_utc();

        let new_message = NewMessage {
            session_id: Some(1),
            source_e164: Some(pn1.clone()),
            source_uuid: None,
            text: String::from("nyt joni ne velat!"),
            timestamp,
            sent: false,
            received: true,
            is_read: true,
            flags: 0,
            attachment: None,
            mime_type: None,
            has_attachment: false,
            outgoing: false,
            is_unidentified: false,
            quote_timestamp: None,
        };

        let (msg, session) = storage.process_message(new_message, None);
        assert_eq!(session.id, sess1.id);

        // Test no extra session was created
        let sessions = storage.fetch_sessions();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, sess1.id);

        assert_eq!(msg.server_timestamp, timestamp);
    }
}

/// This tests code that may potentially be removed after release
/// but it's important as long as we receive messages without ACK
#[rstest]
#[ignore]
#[actix_rt::test]
async fn dev_message_update(storage: impl Future<Output = InMemoryDb>) {
    let (storage, _temp_dir) = storage.await;

    let pn1 = phonenumber::parse(None, "+358501234567").unwrap();
    let session = storage.fetch_or_insert_session_by_phonenumber(&pn1);

    let timestamp = Utc::now().naive_utc();
    // Receive basic message
    let new_message = NewMessage {
        session_id: Some(session.id),
        source_e164: Some(pn1.clone()),
        source_uuid: None,
        text: String::from("nyt joni ne velat!"),
        timestamp,
        sent: false,
        received: true,
        is_read: true,
        flags: 0,
        attachment: None,
        mime_type: None,
        has_attachment: false,
        outgoing: false,
        is_unidentified: false,
        quote_timestamp: None,
    };

    storage.process_message(new_message, None);

    // Though this is tested in other cases, double-check a message exists
    let db_messages = storage.fetch_all_messages(1);
    assert_eq!(db_messages.len(), 1);

    // However, there should have been an attachment
    // which the Go worker would do before `process_message`
    let other_message = NewMessage {
        session_id: Some(session.id),
        source_e164: Some(pn1),
        source_uuid: None,
        text: String::from("nyt joni ne velat!"),
        timestamp,
        sent: false,
        received: true,
        is_read: true,
        flags: 0,
        attachment: Some(String::from("uuid-uuid-uuid-uuid")),
        mime_type: Some(String::from("text/plain")),
        has_attachment: true,
        outgoing: false,
        is_unidentified: false,
        quote_timestamp: None,
    };

    storage.process_message(other_message, None);

    // And all the messages should still be only one message
    let db_messages = storage.fetch_all_messages(1);
    assert_eq!(db_messages.len(), 1);
}

#[rstest]
#[actix_rt::test]
#[should_panic]
async fn process_inbound_group_message_without_sender(storage: impl Future<Output = InMemoryDb>) {
    let (storage, _temp_dir) = storage.await;

    let new_message = NewMessage {
        session_id: None,
        source_e164: None,
        source_uuid: None,
        text: String::from("MSG 1"),
        timestamp: Utc::now().naive_utc(),
        sent: false,
        received: true,
        is_read: true,
        flags: 0,
        attachment: None,
        mime_type: None,
        has_attachment: false,
        outgoing: false,
        is_unidentified: false,
        quote_timestamp: None,
    };

    // Here the client worker will have resolved a group exists
    let group_id = vec![42u8, 126u8, 71u8, 75u8];
    let pn1 = phonenumber::parse(None, "+358501234567").unwrap();
    let pn2 = phonenumber::parse(None, "+358501234568").unwrap();
    let pn3 = phonenumber::parse(None, "+358501234569").unwrap();
    let group = GroupV1 {
        id: group_id,
        name: String::from("Spurdospärde"),
        members: vec![pn1, pn2, pn3],
    };

    let (message_inserted, session_inserted) = storage.process_message(
        new_message,
        Some(storage.fetch_or_insert_session_by_group_v1(&group)),
    );

    // Test a session was created
    let session = storage
        .fetch_session_by_id(session_inserted.id)
        .expect("created session");
    let group = session.unwrap_group_v1();
    assert_eq!(&group.name, ("Spurdospärde"));
    assert_eq!(&group.id, ("2a7e474b"));

    assert_eq!(message_inserted.session_id, session.id);
}

#[rstest]
#[actix_rt::test]
async fn process_outbound_group_message_without_sender(storage: impl Future<Output = InMemoryDb>) {
    let (storage, _temp_dir) = storage.await;

    let new_message = NewMessage {
        session_id: Some(1),
        source_e164: None,
        source_uuid: None,
        text: String::from("MSG 1"),
        timestamp: Utc::now().naive_utc(),
        sent: false,
        received: true,
        is_read: true,
        flags: 0,
        attachment: None,
        mime_type: None,
        has_attachment: false,
        outgoing: true,
        is_unidentified: false,
        quote_timestamp: None,
    };

    // Here the client worker will have resolved a group exists
    let group_id = vec![42u8, 126u8, 71u8, 75u8];
    let pn1 = phonenumber::parse(None, "+358501234567").unwrap();
    let pn2 = phonenumber::parse(None, "+358501234568").unwrap();
    let pn3 = phonenumber::parse(None, "+358501234569").unwrap();
    let group = GroupV1 {
        id: group_id,
        name: String::from("Spurdospärde"),
        members: vec![pn1, pn2, pn3],
    };

    let (message_inserted, session_inserted) = storage.process_message(
        new_message,
        Some(storage.fetch_or_insert_session_by_group_v1(&group)),
    );

    // Test a session was created
    let session = storage
        .fetch_session_by_id(session_inserted.id)
        .expect("created session");
    let group = session.unwrap_group_v1();
    assert_eq!(&group.name, ("Spurdospärde"));
    assert_eq!(&group.id, ("2a7e474b"));

    assert_eq!(message_inserted.session_id, session.id);
}

#[rstest]
#[actix_rt::test]
async fn process_message_with_group(storage: impl Future<Output = InMemoryDb>) {
    let (storage, _temp_dir) = storage.await;

    let pn1 = phonenumber::parse(None, "+358501234567").unwrap();
    let pn2 = phonenumber::parse(None, "+358501234568").unwrap();
    let pn3 = phonenumber::parse(None, "+358501234569").unwrap();
    let new_message = NewMessage {
        session_id: Some(1),
        source_e164: Some(pn1.clone()),
        source_uuid: None,
        text: String::from("MSG 1"),
        timestamp: Utc::now().naive_utc(),
        sent: false,
        received: true,
        is_read: true,
        flags: 0,
        attachment: None,
        mime_type: None,
        has_attachment: false,
        outgoing: false,
        is_unidentified: false,
        quote_timestamp: None,
    };

    // Here the client worker will have resolved a group exists
    let group_id = vec![42u8, 126u8, 71u8, 75u8];
    let group = GroupV1 {
        id: group_id,
        name: String::from("Spurdospärde"),
        members: vec![pn1, pn2, pn3],
    };

    let (message_inserted, session_inserted) = storage.process_message(
        new_message,
        Some(storage.fetch_or_insert_session_by_group_v1(&group)),
    );

    // Test a session was created
    let session = storage
        .fetch_session_by_id(session_inserted.id)
        .expect("created session");
    let group = session.unwrap_group_v1();
    assert_eq!(&group.name, ("Spurdospärde"));
    assert_eq!(&group.id, ("2a7e474b"));

    assert_eq!(message_inserted.session_id, session.id);
}

#[rstest(ext, case("mp4"), case("jpg"), case("jpg"), case("png"), case("txt"))]
#[actix_rt::test]
async fn test_save_attachment(ext: &str) {
    use rand::distributions::Alphanumeric;
    use rand::{Rng, RngCore};

    env_logger::try_init().ok();

    let location = store::temp();
    let rng = rand::thread_rng();

    // Signaling password for REST API
    let password: String = rng.sample_iter(&Alphanumeric).take(24).collect();

    // Signaling key that decrypts the incoming Signal messages
    let mut rng = rand::thread_rng();
    let mut signaling_key = [0u8; 52];
    rng.fill_bytes(&mut signaling_key);
    let signaling_key = signaling_key;

    // Registration ID
    let regid = 12345;

    let storage = Storage::new(
        Arc::new(SignalConfig::default()),
        &location,
        None,
        regid,
        &password,
        signaling_key,
        None,
    )
    .await
    .unwrap();

    // Create content for attachment and write to file
    let content = [1u8; 10];
    let fname = storage
        .save_attachment(
            &storage.path().join("storage").join("attachments"),
            ext,
            &content,
        )
        .await
        .unwrap();

    // Check existence of attachment
    let exists = std::path::Path::new(&fname).exists();

    println!("Looking for {}", fname.to_str().unwrap());
    assert!(exists);

    assert_eq!(
        fname.extension().unwrap(),
        ext,
        "{} <> {}",
        fname.to_str().unwrap(),
        ext
    );
}

#[rstest(
    storage_password,
    case(Some(String::from("some password"))),
    case(None)
)]
#[actix_rt::test]
async fn test_create_and_open_storage(
    storage_password: Option<String>,
) -> Result<(), anyhow::Error> {
    use rand::distributions::Alphanumeric;
    use rand::{Rng, RngCore};

    env_logger::try_init().ok();

    let location = store::temp();
    let rng = rand::thread_rng();

    // Signaling password for REST API
    let password: String = rng.sample_iter(&Alphanumeric).take(24).collect();

    // Signaling key that decrypts the incoming Signal messages
    let mut rng = rand::thread_rng();
    let mut signaling_key = [0u8; 52];
    rng.fill_bytes(&mut signaling_key);
    let signaling_key = signaling_key;

    // Registration ID
    let regid = 12345;

    let storage = Storage::new(
        Arc::new(SignalConfig::default()),
        &location,
        storage_password.as_deref(),
        regid,
        &password,
        signaling_key,
        None,
    )
    .await;
    assert!(storage.is_ok(), "{}", storage.err().unwrap());
    let storage = storage.unwrap();
    assert_eq!(storage.is_encrypted(), storage_password.is_some());

    macro_rules! tests {
        ($storage:ident) => {{
            use libsignal_service::prelude::protocol::IdentityKeyStore;
            // TODO: assert that tables exist
            assert_eq!(password, $storage.signal_password().await?);
            assert_eq!(signaling_key, $storage.signaling_key().await?);
            assert_eq!(regid, $storage.get_local_registration_id(None).await?);

            let (signed, unsigned) = $storage.next_pre_key_ids().await;
            // Unstarted client will have no pre-keys.
            assert_eq!(0, signed);
            assert_eq!(0, unsigned);

            Result::<_, anyhow::Error>::Ok(())
        }};
    }

    tests!(storage)?;
    drop(storage);

    if storage_password.is_some() {
        assert!(
            Storage::open(Arc::new(SignalConfig::default()), &location, None)
                .await
                .is_err(),
            "Storage was not encrypted"
        );
    }

    let storage = Storage::open(
        Arc::new(SignalConfig::default()),
        &location,
        storage_password,
    )
    .await;
    assert!(storage.is_ok(), "{}", storage.err().unwrap());
    let storage = storage.unwrap();

    tests!(storage)?;

    Ok(())
}

#[actix_rt::test]
async fn test_recipient_actions() {
    use rand::distributions::Alphanumeric;
    use rand::{Rng, RngCore};

    env_logger::try_init().ok();

    let location = store::temp();
    let rng = rand::thread_rng();

    // Signaling password for REST API
    let password: String = rng.sample_iter(&Alphanumeric).take(24).collect();

    // Signaling key that decrypts the incoming Signal messages
    let mut rng = rand::thread_rng();
    let mut signaling_key = [0u8; 52];
    rng.fill_bytes(&mut signaling_key);
    let signaling_key = signaling_key;

    // Registration ID
    let regid = 12345;

    let storage = Storage::new(
        Arc::new(SignalConfig::default()),
        &location,
        None,
        regid,
        &password,
        signaling_key,
        None,
    )
    .await;
    assert!(storage.is_ok(), "{}", storage.err().unwrap());
    let mut storage = storage.unwrap();

    // It seems this function is completely unused!
    let tmp_write_dir = tempfile::tempdir().unwrap();
    let tmp_write_file = tmp_write_dir.path().join("tmp.file");
    storage
        .write_file(tmp_write_file, "something")
        .await
        .unwrap();

    let uuid1 = uuid::Uuid::new_v4();

    let recip = storage.fetch_or_insert_recipient_by_uuid(&uuid1.to_string());

    assert_eq!(
        recip.unidentified_access_mode,
        UnidentifiedAccessMode::Unknown
    );
    storage.set_recipient_unidentified(recip.id, UnidentifiedAccessMode::Disabled);
    let recip = storage.fetch_or_insert_recipient_by_uuid(&uuid1.to_string());
    assert_eq!(
        recip.unidentified_access_mode,
        UnidentifiedAccessMode::Disabled
    );
    storage.set_recipient_unidentified(recip.id, UnidentifiedAccessMode::Enabled);
    let recip = storage.fetch_or_insert_recipient_by_uuid(&uuid1.to_string());
    assert_eq!(
        recip.unidentified_access_mode,
        UnidentifiedAccessMode::Enabled
    );
    storage.set_recipient_unidentified(recip.id, UnidentifiedAccessMode::Unrestricted);
    let recip = storage.fetch_or_insert_recipient_by_uuid(&uuid1.to_string());
    assert_eq!(
        recip.unidentified_access_mode,
        UnidentifiedAccessMode::Unrestricted
    );

    let session = storage.fetch_or_insert_session_by_recipient_id(recip.id);
    let ts = NaiveDateTime::parse_from_str("2023-04-01 07:01:32", "%Y-%m-%d %H:%M:%S").unwrap();
    let msg = NewMessage {
        session_id: Some(session.id),
        timestamp: ts,
        sent: true,
        received: true,
        flags: 0,
        attachment: None,
        outgoing: true,
        source_e164: recip.e164.clone(),
        source_uuid: None,
        text: "Hi!".into(),
        is_read: false,
        mime_type: None,
        has_attachment: false,
        is_unidentified: false,
        quote_timestamp: None,
    };

    let msg = storage.create_message(&msg);
    storage.dequeue_message(msg.id, ts, false);

    assert!(!msg.is_read);
    let (_, msg) = storage.mark_message_read(msg.server_timestamp).unwrap();
    assert!(msg.is_read);

    assert!(storage.fetch_message_receipts(msg.id).is_empty());
    storage.mark_message_received(uuid1, msg.server_timestamp, None);
    assert!(!storage.fetch_message_receipts(msg.id).is_empty());

    let reaction = Reaction {
        emoji: Some("❤".into()),
        remove: Some(false),
        target_author_uuid: Some(recip.uuid.unwrap().to_string()),
        target_sent_timestamp: Some(msg.server_timestamp.timestamp_millis() as _),
    };
    let data_msg = DataMessage {
        body: None,
        attachments: [].to_vec(),
        group_v2: None,
        flags: None,
        expire_timer: None,
        profile_key: Some([0].to_vec()),
        timestamp: Some(msg.server_timestamp.timestamp_millis() as _),
        quote: None,
        contact: [].to_vec(),
        preview: [].to_vec(),
        sticker: None,
        required_protocol_version: None,
        is_view_once: None,
        reaction: Some(reaction.clone()),
        delete: None,
        body_ranges: [].to_vec(),
        group_call_update: None,
        payment: None,
        story_context: None,
        gift_badge: None,
    };
    let (m, s) = storage
        .process_reaction(&recip, &data_msg, &reaction)
        .unwrap();
    assert_eq!(m.id, msg.id);
    assert_eq!(s.id, session.id);
    let r = storage.fetch_reactions_for_message(msg.id);
    assert!(!r.is_empty());
    assert_eq!(r.first().unwrap().0.emoji, String::from("❤"));

    let m = storage
        .fetch_last_message_by_session_id_augmented(session.id)
        .unwrap();
    assert_eq!(m.text.as_ref().unwrap(), &msg.text.unwrap());

    assert!(!msg.sending_has_failed);
    storage.fail_message(msg.id);
    let msg = storage.fetch_message_by_id(msg.id).unwrap();
    assert!(msg.sending_has_failed);

    assert_eq!(storage.mark_pending_messages_failed(), 0);

    assert!(storage.fetch_group_sessions().is_empty());
    assert!(storage
        .fetch_session_by_id_augmented(session.id + 1)
        .is_none());
    assert!(storage.fetch_session_by_id_augmented(session.id).is_some());
    assert!(storage.fetch_attachment(42).is_none());
    assert!(storage.fetch_attachments_for_message(msg.id).is_empty());

    assert_eq!(storage.fetch_all_messages_augmented(session.id).len(), 1);

    assert_eq!(storage.fetch_all_sessions_augmented().len(), 1);

    storage.delete_message(msg.id);
    assert!(storage.fetch_message_by_id(msg.id).is_none());
    drop(msg);

    assert_eq!(storage.fetch_all_messages_augmented(session.id).len(), 0);

    storage.delete_session(session.id);
    assert_eq!(storage.fetch_all_sessions_augmented().len(), 0);
}

// XXX: These tests worked back when Storage had the message_handler implemented.
// This has since been moved to ClientActor, and testing that requires Qt-enabled tests.
// https://gitlab.com/rubdos/whisperfish/-/issues/82

// #[rstest]
// fn message_handler_without_group(storage: InMemoryDb) {
//     setup_db(&storage);
//
//     let res = storage.fetch_session(1);
//     assert!(res.is_none());
//
//     let msg = svcmodels::Message {
//         source: String::from("8483"),
//         message: String::from("sup"),
//         attachments: Vec::new(),
//         group: None,
//         timestamp: 0u64,
//         flags: 0u32,
//     };
//
//     storage.message_handler(msg, false, 0);
//
//     // Test a session was created
//     let session = storage
//         .fetch_session(1)
//         .expect("Expected to find session");
//     assert!(!session.is_group);
//
//     // Test a message was created
//     let message = storage
//         .fetch_latest_message()
//         .expect("Expected to find message");
//     assert_eq!(message.source, "8483");
//     assert_eq!(message.sid, session.id);
// }

// #[rstest]
// fn message_handler_leave_group(storage: InMemoryDb) {
//     setup_db(&storage);
//
//     let res = storage.fetch_session(1);
//     assert!(res.is_none());
//
//     let group_id = vec![42u8, 126u8, 71u8, 75u8];
//     let group = svcmodels::Group {
//         id: group_id.clone(),
//         hex_id: hex::encode(group_id.clone()),
//         flags: GROUP_LEAVE_FLAG,
//         name: String::from("Spurdospärde"),
//         members: vec![
//             String::from("Joni"),
//             String::from("Make"),
//             String::from("Spurdoliina"),
//         ],
//         avatar: None,
//     };
//
//     let msg = svcmodels::Message {
//         source: String::from("8483"),
//         message: String::from("Spurdoliina went away or something"),
//         attachments: Vec::new(),
//         group: Some(group),
//         timestamp: 0u64,
//         flags: 0u32,
//     };
//
//     storage.message_handler(msg, false, 0);
//
//     // Test a session was created
//     let session = storage
//         .fetch_session(1)
//         .expect("Expected to find session");
//     assert!(session.is_group);
//
//     // Test a message was created
//     let message = storage
//         .fetch_latest_message()
//         .expect("Expected to find message");
//     assert_eq!(message.source, "8483");
//     assert_eq!(message.message, "Member left group");
//     assert_eq!(message.sid, session.id);
// }

// #[rstest]
// fn message_handler_join_group(storage: InMemoryDb) {
//     setup_db(&storage);
//
//     let res = storage.fetch_session(1);
//     assert!(res.is_none());
//
//     let group_id = vec![42u8, 126u8, 71u8, 75u8];
//     let group = svcmodels::Group {
//         id: group_id.clone(),
//         hex_id: hex::encode(group_id.clone()),
//         flags: GROUP_UPDATE_FLAG,
//         name: String::from("Spurdospärde"),
//         members: vec![String::from("Joni"), String::from("Make")],
//         avatar: None,
//     };
//
//     let msg = svcmodels::Message {
//         source: String::from("8483"),
//         message: String::from("Spurdoliina came back or something"),
//         attachments: Vec::new(),
//         group: Some(group),
//         timestamp: 0u64,
//         flags: 0u32,
//     };
//
//     storage.message_handler(msg, false, 0);
//
//     // Test a session was created
//     let session = storage
//         .fetch_session(1)
//         .expect("Expected to find session");
//     assert!(session.is_group);
//
//     // Test a message was created
//     let message = storage
//         .fetch_latest_message()
//         .expect("Expected to find message");
//     assert_eq!(message.source, "8483");
//     assert_eq!(message.message, "Member joined group");
//     assert_eq!(message.sid, session.id);
// }

// #[rstest]
// fn message_handler_group_attachment_no_save(storage: InMemoryDb) {
//     setup_db(&storage);
//
//     let res = storage.fetch_session(1);
//     assert!(res.is_none());
//
//     let group_id = vec![42u8, 126u8, 71u8, 75u8];
//     let group = svcmodels::Group {
//         id: group_id.clone(),
//         hex_id: hex::encode(group_id.clone()),
//         flags: 0,
//         name: String::from("Spurdospärde"),
//         members: vec![
//             String::from("Joni"),
//             String::from("Make"),
//             String::from("Spurdoliina"),
//         ],
//         avatar: None,
//     };
//
//     let attachment = svcmodels::Attachment::<u8> {
//         reader: 0u8,
//         mime_type: String::from("image/jpg"),
//     };
//
//     let msg = svcmodels::Message {
//         source: String::from("8483"),
//         message: String::from("KIKKI HIIREN KUVA:DDD"),
//         attachments: vec![attachment],
//         group: Some(group),
//         timestamp: 0u64,
//         flags: 0u32,
//     };
//
//     storage.message_handler(msg, false, 0);
//
//     // Test a session was created
//     let session = storage
//         .fetch_session(1)
//         .expect("Expected to find session");
//     assert!(session.is_group);
//
//     // Test a message was created
//     let message = storage
//         .fetch_latest_message()
//         .expect("Expected to find message");
//     assert_eq!(message.source, "8483");
//     assert_eq!(message.message, "KIKKI HIIREN KUVA:DDD");
//     assert_eq!(message.sid, session.id);
//
//     // By default, attachments are not saved, so this should not exist
//     assert!(message.attachment.is_none());
// }
