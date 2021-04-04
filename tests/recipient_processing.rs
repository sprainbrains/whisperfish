use std::future::Future;

use rstest::{fixture, rstest};

mod common;
use common::*;

const E164: &str = "+32474";
const UUID: &str = "abcd-ef123";

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

mod merge_and_fetch {
    use super::*;
    use harbour_whisperfish::store::TrustLevel;

    #[rstest]
    #[actix_rt::test]
    async fn trusted_pair(storage: impl Future<Output = InMemoryDb>) {
        let (storage, _temp_dir) = storage.await;

        let recipient =
            storage.merge_and_fetch_recipient(Some(E164), Some(UUID), TrustLevel::Certain);
        assert_eq!(recipient.e164.as_deref(), Some(E164));
        assert_eq!(recipient.uuid.as_deref(), Some(UUID));
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
