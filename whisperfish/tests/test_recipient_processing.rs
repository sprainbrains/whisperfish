use std::future::Future;

use rstest::{fixture, rstest};

mod common;
use common::*;

const E164: &str = "+32474";
const UUID: &str = "abcd-ef123";
const UUID2: &str = "abcd-ef1234-56789";

#[fixture]
fn storage_with_e164_recipient(
    storage: impl Future<Output = InMemoryDb>,
) -> impl Future<Output = InMemoryDb> {
    use futures::prelude::*;
    storage.map(|(storage, _temp_dir)| {
        storage.fetch_or_insert_recipient_by_e164(E164);

        (storage, _temp_dir)
    })
}

#[fixture]
fn storage_with_uuid_recipient(
    storage: impl Future<Output = InMemoryDb>,
) -> impl Future<Output = InMemoryDb> {
    use futures::prelude::*;
    storage.map(|(storage, _temp_dir)| {
        storage.fetch_or_insert_recipient_by_uuid(UUID);

        (storage, _temp_dir)
    })
}

#[rstest]
#[actix_rt::test]
async fn insert_then_fetch_by_e164(storage: impl Future<Output = InMemoryDb>) {
    let (storage, _temp_dir) = storage.await;

    let recipient1 = storage.fetch_or_insert_recipient_by_e164(E164);
    let recipient2 = storage.fetch_or_insert_recipient_by_e164(E164);
    assert_eq!(recipient1.id, recipient2.id);
    assert_eq!(recipient1.e164.as_deref(), Some(E164));
}

#[rstest]
#[actix_rt::test]
async fn insert_then_fetch_by_uuid(storage: impl Future<Output = InMemoryDb>) {
    let (storage, _temp_dir) = storage.await;

    let recipient1 = storage.fetch_or_insert_recipient_by_uuid(UUID);
    let recipient2 = storage.fetch_or_insert_recipient_by_uuid(UUID);
    assert_eq!(recipient1.id, recipient2.id);
    assert_eq!(recipient1.uuid.as_deref(), Some(UUID));
}

mod merge_and_fetch {
    use super::*;
    use whisperfish::store::TrustLevel;

    #[rstest]
    #[actix_rt::test]
    async fn trusted_pair(storage: impl Future<Output = InMemoryDb>) {
        let (storage, _temp_dir) = storage.await;

        let recipient =
            storage.merge_and_fetch_recipient(Some(E164), Some(UUID), TrustLevel::Certain);
        assert_eq!(recipient.e164.as_deref(), Some(E164));
        assert_eq!(recipient.uuid.as_deref(), Some(UUID));

        // Second call should be a no-op
        let recipient_check =
            storage.merge_and_fetch_recipient(Some(E164), Some(UUID), TrustLevel::Certain);
        assert_eq!(recipient.e164.as_deref(), Some(E164));
        assert_eq!(recipient.uuid.as_deref(), Some(UUID));
        assert_eq!(recipient_check.id, recipient.id);
    }

    #[rstest]
    #[actix_rt::test]
    async fn untrusted_pair(storage: impl Future<Output = InMemoryDb>) {
        let (storage, _temp_dir) = storage.await;

        let recipient =
            storage.merge_and_fetch_recipient(Some(E164), Some(UUID), TrustLevel::Uncertain);
        assert_eq!(recipient.e164.as_deref(), None);
        assert_eq!(recipient.uuid.as_deref(), Some(UUID));
    }

    #[rstest]
    #[actix_rt::test]
    async fn trusted_amend_e164(storage_with_e164_recipient: impl Future<Output = InMemoryDb>) {
        let (storage, _temp_dir) = storage_with_e164_recipient.await;

        let recipient =
            storage.merge_and_fetch_recipient(Some(E164), Some(UUID), TrustLevel::Certain);
        assert_eq!(recipient.e164.as_deref(), Some(E164));
        assert_eq!(recipient.uuid.as_deref(), Some(UUID));

        assert_eq!(storage.fetch_recipients().len(), 1);
    }

    #[rstest]
    #[actix_rt::test]
    async fn untrusted_amend_e164(storage_with_e164_recipient: impl Future<Output = InMemoryDb>) {
        let (storage, _temp_dir) = storage_with_e164_recipient.await;

        let recipient =
            storage.merge_and_fetch_recipient(Some(E164), Some(UUID), TrustLevel::Uncertain);
        assert_eq!(recipient.e164.as_deref(), None);
        assert_eq!(recipient.uuid.as_deref(), Some(UUID));

        // Now check that the e164 still exists separately.
        let recipient_e164 = storage
            .fetch_recipient(Some(E164), None)
            .expect("e164 still in db");
        assert_eq!(recipient_e164.e164.as_deref(), Some(E164));
        assert_eq!(recipient_e164.uuid.as_deref(), None);

        assert_eq!(storage.fetch_recipients().len(), 2);

        let recipient_uuid = storage
            .fetch_recipient(None, Some(UUID))
            .expect("uuid still in db");
        assert_eq!(recipient.id, recipient_uuid.id);
    }

    #[rstest]
    #[actix_rt::test]
    async fn trusted_amend_uuid(storage_with_uuid_recipient: impl Future<Output = InMemoryDb>) {
        let (storage, _temp_dir) = storage_with_uuid_recipient.await;

        let recipient =
            storage.merge_and_fetch_recipient(Some(E164), Some(UUID), TrustLevel::Certain);
        assert_eq!(recipient.e164.as_deref(), Some(E164));
        assert_eq!(recipient.uuid.as_deref(), Some(UUID));

        assert_eq!(storage.fetch_recipients().len(), 1);
    }

    #[rstest]
    #[actix_rt::test]
    async fn untrusted_amend_uuid(storage_with_uuid_recipient: impl Future<Output = InMemoryDb>) {
        let (storage, _temp_dir) = storage_with_uuid_recipient.await;

        let recipient =
            storage.merge_and_fetch_recipient(Some(E164), Some(UUID), TrustLevel::Uncertain);
        assert_eq!(recipient.e164.as_deref(), None);
        assert_eq!(recipient.uuid.as_deref(), Some(UUID));

        // Now check that the e164 does not exist separately.
        assert!(storage.fetch_recipient(Some(E164), None).is_none());

        assert_eq!(storage.fetch_recipients().len(), 1);
    }
}

mod merge_and_fetch_conflicting_recipients {
    use super::*;
    use whisperfish::store::TrustLevel;

    #[rstest]
    #[actix_rt::test]
    async fn trusted_disjunct_recipients(storage: impl Future<Output = InMemoryDb>) {
        let (storage, _temp_dir) = storage.await;

        let r1 = storage.fetch_or_insert_recipient_by_e164(E164);
        let r2 = storage.fetch_or_insert_recipient_by_uuid(UUID);
        // We have two separate recipients.
        assert_ne!(r1.id, r2.id);
        assert_eq!(storage.fetch_recipients().len(), 2);

        // If we now fetch the recipient based on both e164 and uuid, with certainty of their
        // relation,
        // we trigger their merger.
        let recipient =
            storage.merge_and_fetch_recipient(Some(E164), Some(UUID), TrustLevel::Certain);
        assert_eq!(recipient.e164.as_deref(), Some(E164));
        assert_eq!(recipient.uuid.as_deref(), Some(UUID));

        // Now check that the e164/uuid does not exist separately.
        assert_eq!(storage.fetch_recipients().len(), 1);
    }

    #[rstest]
    #[actix_rt::test]
    async fn untrusted_disjunct_recipients(storage: impl Future<Output = InMemoryDb>) {
        let (storage, _temp_dir) = storage.await;

        let r1 = storage.fetch_or_insert_recipient_by_e164(E164);
        let r2 = storage.fetch_or_insert_recipient_by_uuid(UUID);
        // We have two separate recipients.
        assert_ne!(r1.id, r2.id);
        assert_eq!(storage.fetch_recipients().len(), 2);

        // If we now fetch the recipient based on both e164 and uuid, with certainty of their
        // relation,
        // we trigger their merger.
        let recipient =
            storage.merge_and_fetch_recipient(Some(E164), Some(UUID), TrustLevel::Uncertain);
        assert_eq!(recipient.e164.as_deref(), None);
        assert_eq!(recipient.id, r2.id);
        assert_eq!(recipient.uuid.as_deref(), Some(UUID));

        // Now check that the e164 exists separately.
        assert_eq!(storage.fetch_recipients().len(), 2);
    }

    #[rstest]
    #[actix_rt::test]
    async fn trusted_recipient_with_new_uuid(storage: impl Future<Output = InMemoryDb>) {
        let (storage, _temp_dir) = storage.await;

        let r1 = storage.merge_and_fetch_recipient(Some(E164), Some(UUID), TrustLevel::Certain);
        let r2 = storage.fetch_or_insert_recipient_by_uuid(UUID2);
        // We have two separate recipients.
        assert_ne!(r1.id, r2.id);
        assert_eq!(storage.fetch_recipients().len(), 2);
        assert_eq!(r1.e164.as_deref(), Some(E164));
        assert_eq!(r1.uuid.as_deref(), Some(UUID));

        // If we now fetch the recipient based on both e164 and uuid2, with certainty of their
        // relation,
        // we trigger the move of the phone number.
        // XXX Signal Android then marks the former as "needing refresh". Still need to figure out what
        // that is, but it probably checks with the server than indeed the former UUID doesn't
        // exist anymore, and that the data needs to be moved.
        let recipient =
            storage.merge_and_fetch_recipient(Some(E164), Some(UUID2), TrustLevel::Certain);
        assert_eq!(recipient.e164.as_deref(), Some(E164));
        assert_eq!(recipient.uuid.as_deref(), Some(UUID2));

        // Now check that the old recipient still exists.
        assert_eq!(storage.fetch_recipients().len(), 2);

        let recipient = storage
            .fetch_recipient_by_id(r1.id)
            .expect("r1 still exists");
        assert_eq!(recipient.uuid.as_deref(), Some(UUID));
        assert_eq!(recipient.e164.as_deref(), None);
    }

    #[rstest]
    #[actix_rt::test]
    async fn untrusted_recipient_with_new_uuid(storage: impl Future<Output = InMemoryDb>) {
        let (storage, _temp_dir) = storage.await;

        let r1 = storage.merge_and_fetch_recipient(Some(E164), Some(UUID), TrustLevel::Certain);
        let r2 = storage.fetch_or_insert_recipient_by_uuid(UUID2);
        // We have two separate recipients.
        assert_ne!(r1.id, r2.id);
        assert_eq!(storage.fetch_recipients().len(), 2);
        assert_eq!(r1.e164.as_deref(), Some(E164));
        assert_eq!(r1.uuid.as_deref(), Some(UUID));

        // If we now fetch the recipient based on both e164 and uuid2, with uncertainty of their
        // relation,
        // we should get the uuid2 recipient without any other action.
        let recipient =
            storage.merge_and_fetch_recipient(Some(E164), Some(UUID2), TrustLevel::Uncertain);
        assert_eq!(recipient.e164.as_deref(), None);
        assert_eq!(recipient.uuid.as_deref(), Some(UUID2));

        // Now check that the old recipient still exists.
        assert_eq!(storage.fetch_recipients().len(), 2);

        let recipient = storage
            .fetch_recipient_by_id(r1.id)
            .expect("r1 still exists");
        assert_eq!(recipient.uuid.as_deref(), Some(UUID));
        assert_eq!(recipient.e164.as_deref(), Some(E164));
    }
}
