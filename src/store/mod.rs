use std::path::{Path, PathBuf};

use diesel::prelude::*;
use failure::*;

/// Location of the storage.
///
/// Path is for persistent storage.
/// Memory is for running tests or 'incognito' mode.
pub enum StorageLocation<P> {
    Path(P),
    Memory,
}

impl<'a> From<&'a Path> for StorageLocation<&'a Path> {
    fn from(p: &'a Path) -> Self {
        StorageLocation::Path(p)
    }
}

impl From<PathBuf> for StorageLocation<PathBuf> {
    fn from(p: PathBuf) -> Self {
        StorageLocation::Path(p)
    }
}

pub fn memory() -> StorageLocation<PathBuf> {
    StorageLocation::Memory
}

pub fn default_location() -> Result<StorageLocation<PathBuf>, Error> {
    let data_dir = dirs::data_local_dir().ok_or(format_err!("Could not find data directory."))?;

    Ok(StorageLocation::Path(
        data_dir.join("harbour-whisperfish").into(),
    ))
}

impl<P: AsRef<Path>> StorageLocation<P> {
    fn open_db(&self) -> Result<SqliteConnection, Error> {
        let database_url = match self {
            StorageLocation::Memory => ":memory:".into(),
            StorageLocation::Path(p) => p
                .as_ref()
                .join("db")
                .join("harbour-whisperfish.db")
                .to_str()
                .ok_or(format_err!(
                    "path to db contains a non-UTF8 character, please file a bug."
                ))?
                .to_string(),
        };

        Ok(SqliteConnection::establish(&database_url)?)
    }
}

pub struct Storage {
    db: SqliteConnection,
}

impl Storage {
    pub fn open<T: AsRef<Path>>(db_path: &StorageLocation<T>) -> Result<Storage, Error> {
        let db = db_path.open_db()?;

        Ok(Storage { db })
    }

    pub fn open_with_key<T: AsRef<Path>>(
        db_path: &StorageLocation<T>,
        key: [u8; 32],
    ) -> Result<Storage, Error> {
        let db = db_path.open_db()?;

        // Decrypt db
        // XXX we assume all databases to be encrypted.

        Ok(Storage { db })
    }
}
