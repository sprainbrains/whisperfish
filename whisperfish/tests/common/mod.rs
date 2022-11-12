use rstest::fixture;
use std::future::Future;
use whisperfish::store::temp;
use whisperfish::store::{Storage, StorageLocation};

pub type InMemoryDb = (Storage, StorageLocation<tempdir::TempDir>);

/// We do not want to test on a live db, use temporary dir
#[fixture]
#[allow(clippy::manual_async_fn)]
pub fn storage() -> impl Future<Output = InMemoryDb> {
    async {
        let temp = temp();
        (
            Storage::new(&temp, None, 12345, "Some Password", [0; 52], None)
                .await
                .expect("Failed to initalize storage"),
            temp,
        )
    }
}
