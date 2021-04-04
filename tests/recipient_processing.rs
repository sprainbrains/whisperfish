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
    storage.map(|(storage, _temp_dir)| (storage, _temp_dir))
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
}
