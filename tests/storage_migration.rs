//! These integration tests check whether an old storage implementation can be opened. The old
//! storage implementation is stripped down to write files (e.g., identity keys, sessions,
//! attachments, etc.), and opening the database. We don't test any migrations of messages, etc.
//! here. The created storage is then read with the current functions.
//!
//! Currently the storage implementation in `current_storage` is at git commit
//! e8ef69ba76b5f40fc149bf1c240df99b62f19b60. Be aware that only necessary parts were copied that
//! were changed in later commits.

use harbour_whisperfish::store as current_storage;

use libsignal_service::prelude::protocol::IdentityKeyStore;
use rstest::rstest;
use std::ops::Deref;

async fn create_old_storage(
    storage_password: Option<&str>,
    path: &current_storage::StorageLocation<tempdir::TempDir>,
) -> current_storage::Storage {
    use rand::{Rng, RngCore};
    let rng = rand::thread_rng();

    // Signaling password for REST API
    let password: String = rng
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(24)
        .collect();

    // Signaling key that decrypts the incoming Signal messages
    let mut rng = rand::thread_rng();
    let mut signaling_key = [0u8; 52];
    rng.fill_bytes(&mut signaling_key);
    let signaling_key = signaling_key;

    // Registration ID
    let regid: u32 = 12345;

    let storage =
        current_storage::Storage::new(path, storage_password, regid, &password, signaling_key)
            .await
            .unwrap();

    storage
}

async fn open_storage(
    storage_password: Option<String>,
    path: &harbour_whisperfish::store::StorageLocation<std::path::PathBuf>,
) -> harbour_whisperfish::store::Storage {
    let storage = harbour_whisperfish::store::Storage::open(path, storage_password)
        .await
        .unwrap();

    storage
}

fn create_random_protocol_address() -> libsignal_service::prelude::protocol::ProtocolAddress {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let user_id = uuid::Uuid::new_v4();
    let device_id = rng.gen_range(2, 20);

    libsignal_service::prelude::protocol::ProtocolAddress::new(user_id.to_string(), device_id)
}

fn create_random_identity_key() -> libsignal_service::prelude::protocol::IdentityKey {
    let mut rng = rand::thread_rng();

    let key_pair = libsignal_service::prelude::protocol::IdentityKeyPair::generate(&mut rng);

    *key_pair.identity_key()
}

#[rstest(
    storage_password,
    case(Some(String::from("some password"))),
    case(None)
)]
#[actix_rt::test]
async fn read_own_identity_key(storage_password: Option<String>) {
    let location = current_storage::temp();
    let storage = create_old_storage(storage_password.as_deref(), &location).await;

    // Get own identity key
    let own_identity_key_1 = storage.get_identity_key_pair(None).await.unwrap();

    // Drop storage
    drop(storage);

    // Open storage with new implementation
    let location: harbour_whisperfish::store::StorageLocation<std::path::PathBuf> =
        location.deref().to_path_buf().into();
    let storage = open_storage(storage_password, &location).await;

    // Get own identity key
    let own_identity_key_2 = storage.get_identity_key_pair(None).await.unwrap();

    // Test equality
    assert_eq!(
        own_identity_key_1.serialize(),
        own_identity_key_2.serialize()
    );
}

#[rstest(
    storage_password,
    case(Some(String::from("some password"))),
    case(None)
)]
#[actix_rt::test]
async fn read_regid(storage_password: Option<String>) {
    let location = current_storage::temp();
    let storage = create_old_storage(storage_password.as_deref(), &location).await;

    // Get own identity key
    let regid_1 = storage.get_local_registration_id(None).await.unwrap();

    // Drop storage
    drop(storage);

    // Open storage with new implementation
    let location: harbour_whisperfish::store::StorageLocation<std::path::PathBuf> =
        location.deref().to_path_buf().into();
    let storage = open_storage(storage_password, &location).await;

    // Get own identity key
    let regid_2 = storage.get_local_registration_id(None).await.unwrap();

    // Test equality
    assert_eq!(regid_1, regid_2);
}

#[rstest(
    storage_password,
    case(Some(String::from("some password"))),
    case(None)
)]
#[actix_rt::test]
async fn read_signal_password(storage_password: Option<String>) {
    let location = current_storage::temp();
    let storage = create_old_storage(storage_password.as_deref(), &location).await;

    // Get own identity key
    let value_1 = storage.signal_password().await.unwrap();

    // Drop storage
    drop(storage);

    // Open storage with new implementation
    let location: harbour_whisperfish::store::StorageLocation<std::path::PathBuf> =
        location.deref().to_path_buf().into();
    let storage = open_storage(storage_password, &location).await;

    // Get own identity key
    let value_2 = storage.signal_password().await.unwrap();

    // Test equality
    assert_eq!(value_1, value_2);
}

#[rstest(
    storage_password,
    case(Some(String::from("some password"))),
    case(None)
)]
#[actix_rt::test]
async fn read_signaling_key(storage_password: Option<String>) {
    let location = current_storage::temp();
    let storage = create_old_storage(storage_password.as_deref(), &location).await;

    // Get own identity key
    let value_1 = storage.signaling_key().await.unwrap();

    // Drop storage
    drop(storage);

    // Open storage with new implementation
    let location: harbour_whisperfish::store::StorageLocation<std::path::PathBuf> =
        location.deref().to_path_buf().into();
    let storage = open_storage(storage_password, &location).await;

    // Get own identity key
    let value_2 = storage.signaling_key().await.unwrap();

    // Test equality
    assert_eq!(value_1, value_2);
}

#[rstest(
    storage_password,
    case(Some(String::from("some password"))),
    case(None)
)]
#[actix_rt::test]
async fn read_other_identity_key(storage_password: Option<String>) {
    let location = current_storage::temp();
    let mut storage = create_old_storage(storage_password.as_deref(), &location).await;

    // Create new identity key
    let addr = create_random_protocol_address();
    let key = create_random_identity_key();

    // Store identity key
    storage.save_identity(&addr, &key, None).await.unwrap();

    // Drop storage
    drop(storage);

    // Open storage with new implementation
    let location: harbour_whisperfish::store::StorageLocation<std::path::PathBuf> =
        location.deref().to_path_buf().into();
    let storage = open_storage(storage_password, &location).await;

    // Get saved identity key
    let key_2 = storage.get_identity(&addr, None).await.unwrap().unwrap();

    // Test equality
    assert_eq!(key, key_2);
}

/// These storages were initialized in June 2022, while moving the identity and session store into the SQLite database.
///
/// https://gitlab.com/whisperfish/whisperfish/-/merge_requests/249
#[rstest]
#[case("tests/storage_migration/without-password-2022-06".into(), None)]
#[case("tests/storage_migration/with-password-123456-2022-06".into(), Some("123456".into()))]
#[actix_rt::test]
async fn test_2022_06_migration(
    #[case] path: std::path::PathBuf,
    #[case] storage_password: Option<String>,
) {
    let storage = harbour_whisperfish::store::Storage::open(&path.into(), storage_password)
        .await
        .expect("open older storage");
    let migration =
        harbour_whisperfish::worker::client::migrations::session_to_db::SessionStorageMigration(
            storage,
        );
    migration.execute().await;
    // XXX now test that the sessions and identities are still there.
}
