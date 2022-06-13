use actix::prelude::*;
use libsignal_service::prelude::protocol::{
    IdentityKeyStore, ProtocolAddress, SessionStore, SessionStoreExt,
};
use std::io;
use std::path::Path;

use libsignal_service::prelude::protocol::{self, Context};
use protocol::IdentityKeyPair;
use protocol::SignalProtocolError;

use super::*;

mod quirk;

#[derive(Message)]
#[rtype(result = "()")]
pub struct MoveSessionsToDatabase;

struct SessionStorageMigration(Storage);
impl std::ops::Deref for SessionStorageMigration {
    type Target = Storage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for SessionStorageMigration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Handler<MoveSessionsToDatabase> for ClientActor {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, _: MoveSessionsToDatabase, _ctx: &mut Self::Context) -> Self::Result {
        let storage = self.storage.clone().expect("initialized storage");

        let proc = async move {
            let migration = SessionStorageMigration(storage.clone());

            if storage.path().join("storage").join("sessions").exists() {
                migration.migrate_sessions().await;
            }

            if storage.path().join("storage").join("identity").exists() {
                migration.migrate_identities().await;
            }
        };

        std::pin::Pin::from(Box::new(proc))
    }
}

fn convert_io_error(e: io::Error) -> SignalProtocolError {
    // XXX can probably be better, but currently this is only used in session_delete and
    // identity_delete
    SignalProtocolError::InvalidArgument(format!("IO error {}", e))
}

fn addr_to_path_component<'a>(addr: &'a (impl AsRef<[u8]> + ?Sized + 'a)) -> &'a str {
    let addr: &'a [u8] = addr.as_ref();
    let addr = if addr[0] == b'+' { &addr[1..] } else { addr };
    std::str::from_utf8(addr).expect("address in valid UTF8")
}

impl SessionStorageMigration {
    async fn migrate_sessions(&self) {}

    async fn migrate_identities(&self) {}

    fn session_path(&self, addr: &ProtocolAddress) -> PathBuf {
        let recipient_id = addr_to_path_component(addr.name());

        self.0.path().join("storage").join("sessions").join(format!(
            "{}_{}",
            recipient_id,
            addr.device_id()
        ))
    }

    fn identity_path(&self, addr: &ProtocolAddress) -> PathBuf {
        let recipient_id = addr_to_path_component(addr.name());

        self.0
            .path()
            .join("storage")
            .join("identity")
            .join(format!("remote_{}", recipient_id,))
    }

    async fn get_identity_key_pair(
        &self,
        _: Context,
    ) -> Result<IdentityKeyPair, SignalProtocolError> {
        log::trace!("Reading own identity key pair");
        let _lock = self.protocol_store.read().await;

        let path = self
            .path()
            .join("storage")
            .join("identity")
            .join("identity_key");
        let identity_key_pair = {
            use std::convert::TryFrom;
            let mut buf = self.read_file(path).await.map_err(|e| {
                SignalProtocolError::InvalidArgument(format!("Cannot read own identity key {}", e))
            })?;
            buf.insert(0, quirk::DJB_TYPE);
            let public = IdentityKey::decode(&buf[0..33])?;
            let private = PrivateKey::try_from(&buf[33..])?;
            IdentityKeyPair::new(public, private)
        };
        Ok(identity_key_pair)
    }

    async fn is_trusted_identity(
        &self,
        addr: &ProtocolAddress,
        key: &IdentityKey,
        // XXX
        _direction: Direction,
        _ctx: Context,
    ) -> Result<bool, SignalProtocolError> {
        let _lock = self.protocol_store.read().await;

        if let Some(trusted_key) = self.read_identity_key_file(addr).await? {
            Ok(trusted_key == *key)
        } else {
            // Trust on first use
            Ok(true)
        }
    }

    async fn get_identity(
        &self,
        addr: &ProtocolAddress,
        _: Context,
    ) -> Result<Option<IdentityKey>, SignalProtocolError> {
        let _lock = self.protocol_store.read().await;

        self.read_identity_key_file(addr).await
    }

    async fn load_session(
        &self,
        addr: &ProtocolAddress,
        _: Context,
    ) -> Result<Option<SessionRecord>, SignalProtocolError> {
        let path = self.session_path(addr);

        log::trace!("Loading session for {:?} from {:?}", addr, path);
        let _lock = self.protocol_store.read().await;

        let buf = match self.read_file(&path).await {
            Ok(buf) => quirk::session_from_0_5(&buf)?,
            Err(e) if !path.exists() => {
                log::trace!(
                    "Returning None session because session file does not exist ({})",
                    e
                );
                return Ok(None);
            }
            Err(e) => {
                log::error!(
                    "Problem reading session: {}.  Returning empty session, but here be dragons.",
                    e
                );
                return Ok(None);
            }
        };

        Ok(Some(SessionRecord::deserialize(&buf)?))
    }

    async fn store_session(
        &mut self,
        addr: &ProtocolAddress,
        session: &protocol::SessionRecord,
        _: Context,
    ) -> Result<(), SignalProtocolError> {
        let path = self.session_path(addr);

        log::trace!("Storing session for {:?} at {:?}", addr, path);
        let _lock = self.protocol_store.write().await;

        let quirked = quirk::session_to_0_5(&session.serialize()?)?;
        self.write_file(path, quirked).await.unwrap();
        Ok(())
    }

    #[allow(dead_code)]
    async fn contains_session(
        &self,
        addr: &ProtocolAddress,
        _: Context,
    ) -> Result<bool, SignalProtocolError> {
        let _lock = self.protocol_store.read().await;

        let path = self.session_path(addr);
        Ok(path.is_file())
    }

    async fn get_sub_device_sessions(&self, addr: &str) -> Result<Vec<u32>, SignalProtocolError> {
        log::trace!("Looking for sub_device sessions for {}", addr);
        let _lock = self.protocol_store.read().await;

        let addr = addr_to_path_component(addr).as_bytes();

        let session_dir = self.path().join("storage").join("sessions");

        let ids = std::fs::read_dir(session_dir)
            .expect("initialized storage")
            .filter_map(|entry| {
                let entry = entry.expect("directory listing");
                if !entry.path().is_file() {
                    log::warn!("Non-file session entry: {:?}. Skipping", entry);
                    return None;
                }

                // XXX: *maybe* Signal could become a cross-platform desktop app.
                use std::os::unix::ffi::OsStrExt;
                let name = entry.file_name();
                let name = name.as_os_str().as_bytes();

                if name.len() < addr.len() + 2 {
                    return None;
                }

                if &name[..addr.len()] == addr {
                    if name[addr.len()] != b'_' {
                        log::warn!("Weird session directory entry: {:?}. Skipping", entry);
                        return None;
                    }
                    // skip underscore
                    let id = std::str::from_utf8(&name[(addr.len() + 1)..]).ok()?;
                    id.parse().ok()
                } else {
                    None
                }
            })
            .filter(|id| *id != libsignal_service::push_service::DEFAULT_DEVICE_ID)
            .collect();

        Ok(ids)
    }

    async fn delete_session(&self, addr: &ProtocolAddress) -> Result<(), SignalProtocolError> {
        let _lock = self.protocol_store.write().await;

        let path = self.session_path(addr);
        std::fs::remove_file(path).map_err(|e| {
            log::debug!(
                "Could not delete session {}, assuming non-existing: {}",
                addr.to_string(),
                e
            );
            SignalProtocolError::SessionNotFound(addr.clone())
        })?;
        Ok(())
    }

    async fn delete_all_sessions(&self, addr: &str) -> Result<usize, SignalProtocolError> {
        log::warn!("Deleting all sessions for {}", addr);
        let _lock = self.protocol_store.write().await;

        let addr = addr_to_path_component(addr).as_bytes();

        let session_dir = self.path().join("storage").join("sessions");

        let entries = std::fs::read_dir(session_dir)
            .expect("initialized storage")
            .filter_map(|entry| {
                let entry = entry.expect("directory listing");
                if !entry.path().is_file() {
                    log::warn!("Non-file session entry: {:?}. Skipping", entry);
                    return None;
                }

                // XXX: *maybe* Signal could become a cross-platform desktop app.
                use std::os::unix::ffi::OsStrExt;
                let name = entry.file_name();
                let name = name.as_os_str().as_bytes();

                log::trace!("parsing {:?}", entry);

                if name.len() < addr.len() + 2 {
                    log::trace!("filename {:?} not long enough", entry);
                    return None;
                }

                if &name[..addr.len()] == addr {
                    if name[addr.len()] != b'_' {
                        log::warn!("Weird session directory entry: {:?}. Skipping", entry);
                        return None;
                    }
                    // skip underscore
                    let id = std::str::from_utf8(&name[(addr.len() + 1)..]).ok()?;
                    let _: u32 = id.parse().ok()?;
                    Some(entry.path())
                } else {
                    log::trace!("filename {:?} without prefix match", entry);
                    None
                }
            });

        let mut count = 0;
        for entry in entries {
            std::fs::remove_file(entry).map_err(convert_io_error)?;
            count += 1;
        }

        Ok(count)
    }

    #[deprecated]
    async fn read_identity_key_file(
        &self,
        addr: &ProtocolAddress,
    ) -> Result<Option<IdentityKey>, SignalProtocolError> {
        let path = self.identity_path(addr);
        if path.is_file() {
            let buf = self.read_file(path).await.expect("read identity key");
            match buf.len() {
                // Old format
                32 => Ok(Some(
                    protocol::PublicKey::from_djb_public_key_bytes(&buf)?.into(),
                )),
                // New format
                33 => Ok(Some(IdentityKey::decode(&buf)?)),
                _ => Err(SignalProtocolError::InvalidArgument(format!(
                    "Identity key has length {}, expected 32 or 33",
                    buf.len()
                ))),
            }
        } else {
            Ok(None)
        }
    }
}
