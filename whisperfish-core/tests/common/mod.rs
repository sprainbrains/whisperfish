use rstest::fixture;
use std::future::Future;
use std::sync::Arc;
use whisperfish_core::config::SignalConfig;
use whisperfish_core::store::temp;
use whisperfish_core::store::{Storage, StorageLocation};

pub type InMemoryDb = (Storage, StorageLocation<tempfile::TempDir>);

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
