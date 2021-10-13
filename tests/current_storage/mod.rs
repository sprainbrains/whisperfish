use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::Context;

use libsignal_service::groups_v2::InMemoryCredentialsCache;
use libsignal_service::prelude::protocol::*;
use libsignal_service::prelude::*;
use parking_lot::ReentrantMutex;

use diesel::prelude::*;

mod protocol_store;
use protocol_store::ProtocolStore;

pub use harbour_whisperfish::store::temp;
/// Location of the storage.
///
/// Path is for persistent storage.
/// Memory is for running tests or 'incognito' mode.
pub use harbour_whisperfish::store::StorageLocation;

/// Storage Module
#[derive(Clone)]
pub struct Storage {
    pub db: Arc<AssertUnwindSafe<ReentrantMutex<SqliteConnection>>>,
    // aesKey + macKey
    keys: Option<[u8; 16 + 20]>,
    pub(crate) protocol_store: Arc<tokio::sync::RwLock<ProtocolStore>>,
    credential_cache: Arc<Mutex<InMemoryCredentialsCache>>,
    path: PathBuf,
}

// Cannot borrow password/salt because threadpool requires 'static...
async fn derive_storage_key(
    password: String,
    salt_path: PathBuf,
) -> Result<[u8; 16 + 20], anyhow::Error> {
    use actix_threadpool::BlockingError;
    use std::io::Read;

    actix_threadpool::run(move || -> Result<_, anyhow::Error> {
        let mut salt_file = std::fs::File::open(salt_path).context("Cannot open salt file")?;
        let mut salt = [0u8; 8];
        anyhow::ensure!(salt_file.read(&mut salt)? == 8, "salt file not 8 bytes");

        let mut key = [0u8; 16 + 20];
        // Please don't blame me, I'm only the implementer.
        pbkdf2::pbkdf2::<hmac::Hmac<sha1::Sha1>>(password.as_bytes(), &salt, 1024, &mut key);
        log::trace!("Computed the key, salt was {:?}", salt);

        Ok(key)
    })
    .await
    .map_err(|e| match e {
        BlockingError::Canceled => anyhow::anyhow!("Threadpool Canceled"),
        BlockingError::Error(e) => e,
    })
}

// Cannot borrow password/salt because threadpool requires 'static...
async fn derive_db_key(password: String, salt_path: PathBuf) -> Result<[u8; 32], anyhow::Error> {
    use actix_threadpool::BlockingError;
    use std::io::Read;

    actix_threadpool::run(move || -> Result<_, anyhow::Error> {
        let mut salt_file = std::fs::File::open(salt_path)?;
        let mut salt = [0u8; 8];
        anyhow::ensure!(salt_file.read(&mut salt)? == 8, "salt file not 8 bytes");

        let params = scrypt::Params::new(14, 8, 1)?;
        let mut key = [0u8; 32];
        scrypt::scrypt(password.as_bytes(), &salt, &params, &mut key)?;
        log::trace!("Computed the key, salt was {:?}", salt);
        Ok(key)
    })
    .await
    .map_err(|e| match e {
        BlockingError::Canceled => anyhow::anyhow!("Threadpool Canceled"),
        BlockingError::Error(e) => e,
    })
}

fn write_file_sync_unencrypted(path: PathBuf, contents: &[u8]) -> Result<(), anyhow::Error> {
    log::trace!("Writing unencrypted file {:?}", path);

    use std::io::Write;
    let mut file = std::fs::File::create(&path)?;
    file.write_all(contents)?;

    Ok(())
}

fn write_file_sync_encrypted(
    keys: [u8; 16 + 20],
    path: PathBuf,
    contents: &[u8],
) -> Result<(), anyhow::Error> {
    log::trace!("Writing encrypted file {:?}", path);

    // Generate random IV
    use rand::RngCore;
    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);

    // Encrypt
    use aes::Aes128;
    use block_modes::block_padding::Pkcs7;
    use block_modes::{BlockMode, Cbc};
    let ciphertext = {
        let cipher = Cbc::<Aes128, Pkcs7>::new_from_slices(&keys[0..16], &iv)
            .context("CBC initialization error")?;
        cipher.encrypt_vec(contents)
    };

    let mac = {
        use hmac::{Hmac, Mac, NewMac};
        use sha2::Sha256;
        // Verify HMAC SHA256, 32 last bytes
        let mut mac = Hmac::<Sha256>::new_from_slice(&keys[16..])
            .map_err(|_| anyhow::anyhow!("MAC keylength error"))?;
        mac.update(&iv);
        mac.update(&ciphertext);
        mac.finalize().into_bytes()
    };

    // Write iv, ciphertext, mac
    use std::io::Write;
    let mut file = std::fs::File::create(&path)?;
    file.write_all(&iv)?;
    file.write_all(&ciphertext)?;
    file.write_all(&mac)?;

    Ok(())
}

fn write_file_sync(
    keys: Option<[u8; 16 + 20]>,
    path: PathBuf,
    contents: &[u8],
) -> Result<(), anyhow::Error> {
    match keys {
        Some(keys) => write_file_sync_encrypted(keys, path, contents),
        None => write_file_sync_unencrypted(path, contents),
    }
}

async fn write_file(
    keys: Option<[u8; 16 + 20]>,
    path: PathBuf,
    contents: Vec<u8>,
) -> Result<(), anyhow::Error> {
    actix_threadpool::run(move || write_file_sync(keys, path, &contents)).await?;
    Ok(())
}

fn load_file_sync_unencrypted(path: PathBuf) -> Result<Vec<u8>, anyhow::Error> {
    log::trace!("Opening unencrypted file {:?}", path);
    let contents = std::fs::read(&path)?;
    let count = contents.len();
    log::trace!("Read {:?}, {} bytes", path, count);
    Ok(contents)
}

fn load_file_sync_encrypted(keys: [u8; 16 + 20], path: PathBuf) -> Result<Vec<u8>, anyhow::Error> {
    // XXX This is *full* of bad practices.
    // Let's try to migrate to nacl or something alike in the future.

    log::trace!("Opening encrypted file {:?}", path);
    let mut contents = std::fs::read(&path)?;
    let count = contents.len();

    log::trace!("Read {:?}, {} bytes", path, count);
    anyhow::ensure!(count >= 16 + 32, "File smaller than cryptographic overhead");

    let (iv, contents) = contents.split_at_mut(16);
    let count = count - 16;
    let (contents, mac) = contents.split_at_mut(count - 32);

    {
        use hmac::{Hmac, Mac, NewMac};
        use sha2::Sha256;
        // Verify HMAC SHA256, 32 last bytes
        let mut verifier = Hmac::<Sha256>::new_from_slice(&keys[16..])
            .map_err(|_| anyhow::anyhow!("MAC keylength error"))?;
        verifier.update(iv);
        verifier.update(contents);
        verifier
            .verify(mac)
            .map_err(|_| anyhow::anyhow!("MAC error"))?;
    }

    use aes::Aes128;
    use block_modes::block_padding::Pkcs7;
    use block_modes::{BlockMode, Cbc};
    // Decrypt password
    let cipher = Cbc::<Aes128, Pkcs7>::new_from_slices(&keys[0..16], iv)
        .context("CBC initialization error")?;
    Ok(cipher
        .decrypt(contents)
        .context("AES CBC decryption error")?
        .to_owned())
}

fn load_file_sync(keys: Option<[u8; 16 + 20]>, path: PathBuf) -> Result<Vec<u8>, anyhow::Error> {
    match keys {
        Some(keys) => load_file_sync_encrypted(keys, path),
        None => load_file_sync_unencrypted(path),
    }
}

async fn load_file(keys: Option<[u8; 16 + 20]>, path: PathBuf) -> Result<Vec<u8>, anyhow::Error> {
    let contents = actix_threadpool::run(move || load_file_sync(keys, path)).await?;

    Ok(contents)
}

impl Storage {
    fn scaffold_directories(root: impl AsRef<Path>) -> Result<(), anyhow::Error> {
        let root = root.as_ref();

        let directories = [
            root.to_path_buf() as PathBuf,
            root.join("db"),
            root.join("storage"),
            root.join("storage").join("identity"),
            root.join("storage").join("attachments"),
            root.join("storage").join("sessions"),
            root.join("storage").join("prekeys"),
            root.join("storage").join("signed_prekeys"),
            root.join("storage").join("groups"),
        ];

        for dir in &directories {
            if dir.exists() {
                if dir.is_dir() {
                    continue;
                } else {
                    anyhow::bail!(
                        "Trying to create directory {:?}, but already exists as non-directory.",
                        dir
                    );
                }
            }
            std::fs::create_dir(dir)?;
        }
        Ok(())
    }

    /// Writes (*overwrites*) a new Storage object to the provided path.
    pub async fn new<T: AsRef<Path>>(
        db_path: &StorageLocation<T>,
        password: Option<&str>,
        regid: u32,
        http_password: &str,
        signaling_key: [u8; 52],
    ) -> Result<Storage, anyhow::Error> {
        let path: &Path = std::ops::Deref::deref(db_path);

        log::info!("Creating directory structure");
        Self::scaffold_directories(path)?;

        // 1. Generate both salts if needed
        let storage_salt_path = path.join("storage").join("salt");
        if password != None {
            let db_salt_path = path.join("db").join("salt");

            use rand::RngCore;
            log::info!("Generating salts");
            let mut db_salt = [0u8; 8];
            let mut storage_salt = [0u8; 8];
            let mut rng = rand::thread_rng();
            rng.fill_bytes(&mut db_salt);
            rng.fill_bytes(&mut storage_salt);

            std::fs::write(&db_salt_path, db_salt)?;
            std::fs::write(&storage_salt_path, storage_salt)?;
        }

        // 2. Open DB
        let db = Self::open_db(db_path, path, password).await?;

        // 3. initialize protocol store
        let keys = match password {
            None => None,
            Some(pass) => Some(derive_storage_key(pass.to_string(), storage_salt_path).await?),
        };

        let identity_key_pair = protocol::IdentityKeyPair::generate(&mut rand::thread_rng());

        let protocol_store =
            ProtocolStore::store_with_key(keys, path, regid, identity_key_pair).await?;

        // 4. save http password and signaling key
        let identity_path = path.join("storage").join("identity");
        write_file(
            keys,
            identity_path.join("http_password"),
            http_password.as_bytes().into(),
        )
        .await?;
        write_file(
            keys,
            identity_path.join("http_signaling_key"),
            signaling_key.to_vec(),
        )
        .await?;

        Ok(Storage {
            db: Arc::new(AssertUnwindSafe(ReentrantMutex::new(db))),
            keys,
            protocol_store: Arc::new(tokio::sync::RwLock::new(protocol_store)),
            credential_cache: Arc::new(Mutex::new(InMemoryCredentialsCache::default())),
            path: path.to_path_buf(),
        })
    }

    async fn open_db<T: AsRef<Path>>(
        db_path: &StorageLocation<T>,
        path: &Path,
        password: Option<&str>,
    ) -> Result<SqliteConnection, anyhow::Error> {
        log::info!("Opening DB");
        let db = db_path.open_db()?;

        if password != None {
            log::info!("Setting DB encryption");

            let db_salt_path = path.join("db").join("salt");
            let db_key = derive_db_key(password.unwrap().to_string(), db_salt_path);

            db.execute(&format!(
                "PRAGMA key = \"x'{}'\";",
                hex::encode(db_key.await?)
            ))?;
            db.execute("PRAGMA cipher_page_size = 4096;")?;
        }

        // From the sqlcipher manual:
        // -- if this throws an error, the key was incorrect. If it succeeds and returns a numeric value, the key is correct;
        db.execute("SELECT count(*) FROM sqlite_master;")?;
        // XXX: Do we have to signal somehow that the password was wrong?
        //      Offer retries?

        Ok(db)
    }

    /// Asynchronously loads the signal HTTP password from storage and decrypts it.
    pub async fn signal_password(&self) -> Result<String, anyhow::Error> {
        let contents = self
            .load_file(
                self.path
                    .join("storage")
                    .join("identity")
                    .join("http_password"),
            )
            .await?;
        Ok(String::from_utf8(contents)?)
    }

    /// Asynchronously loads the base64 encoded signaling key.
    pub async fn signaling_key(&self) -> Result<[u8; 52], anyhow::Error> {
        let v = self
            .load_file(
                self.path
                    .join("storage")
                    .join("identity")
                    .join("http_signaling_key"),
            )
            .await?;
        anyhow::ensure!(v.len() == 52, "Signaling key is 52 bytes");
        let mut out = [0u8; 52];
        out.copy_from_slice(&v);
        Ok(out)
    }

    async fn load_file(&self, path: PathBuf) -> Result<Vec<u8>, anyhow::Error> {
        load_file(self.keys, path).await
    }
}
