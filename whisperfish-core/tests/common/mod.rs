use rstest::fixture;
use std::future::Future;
use std::sync::Arc;
use uuid::Uuid;
use whisperfish_core::config::SignalConfig;
use whisperfish_core::store::observer::{Event, Interest, Observatory};
use whisperfish_core::store::temp;
use whisperfish_core::store::{Storage, StorageLocation};

#[derive(Default)]
pub struct DummyObservatory;

impl Observatory for DummyObservatory {
    type Subscriber = ();

    fn register(&self, _id: Uuid, _interests: Vec<Interest>, _subscriber: Self::Subscriber) {}

    fn update_interests(&self, _handle: Uuid, _interests: Vec<Interest>) {}

    fn distribute_event(&self, _event: Event) {}
}

pub type SimpleStorage = Storage<DummyObservatory>;

pub type InMemoryDb = (SimpleStorage, StorageLocation<tempfile::TempDir>);

/// We do not want to test on a live db, use temporary dir
#[fixture]
#[allow(clippy::manual_async_fn)]
pub fn storage() -> impl Future<Output = InMemoryDb> {
    async {
        let temp = temp();
        (
            Storage::new(
                // XXX add tempdir to this cfg
                Arc::new(SignalConfig::default()),
                &temp,
                None,
                12345,
                12346,
                "Some Password",
                [0; 52],
                None,
                None,
            )
            .await
            .expect("Failed to initalize storage"),
            temp,
        )
    }
}
