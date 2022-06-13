use actix::prelude::*;
use libsignal_service::prelude::protocol::{
    IdentityKeyStore, ProtocolAddress, SessionStore, SessionStoreExt,
};
use std::io;

use libsignal_service::prelude::protocol::{self, Context};
use protocol::IdentityKeyPair;
use protocol::SignalProtocolError;

use crate::store::orm::SessionRecord;

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

fn option_warn<T>(o: Option<T>, s: &'static str) -> Option<T> {
    if o.is_none() {
        log::warn!("{}", s)
    }
    o
}

impl SessionStorageMigration {
    async fn migrate_sessions(&self) {
        let session_dir = self.path().join("storage").join("sessions");

        let sessions = std::fs::read_dir(session_dir)
            // XXX: actually, storage will stop initializing this.
            .expect("initialized storage")
            // Parse the session file names
            .filter_map(|entry| {
                let entry = entry.expect("directory listing");
                if !entry.path().is_file() {
                    log::warn!("Non-file session entry: {:?}. Skipping", entry);
                    return None;
                }

                // XXX: *maybe* Signal could become a cross-platform desktop app.
                //      Issue #77
                use std::os::unix::ffi::OsStrExt;
                let name = entry.file_name();
                let name = name.as_os_str().as_bytes();

                if name.len() < 3 {
                    log::warn!(
                        "Strange session name; skipping ({})",
                        String::from_utf8_lossy(name)
                    );
                    return None;
                }
                let name = option_warn(
                    std::str::from_utf8(name).ok(),
                    "non-UTF8 session name; skipping",
                )?;

                log::info!("Migrating session {}", name);

                // Parse: session file consists of ADDR + _ + ID
                let mut split = name.split('_');
                let name = option_warn(split.next(), "no session name; skipping")?;
                let id = option_warn(split.next(), "no session id; skipping")?;
                let id: u32 = option_warn(id.parse().ok(), "unparseable session id")?;
                Some(ProtocolAddress::new(name.to_string(), id))
            });

        // Now read the files, put them in the database, and remove the file
        for addr in sessions {
            let path = self.session_path(&addr);

            log::trace!("Loading session for {:?} from {:?}", addr, path);
            let _lock = self.protocol_store.read().await;

            let buf = match self.read_file(&path).await {
                Ok(buf) => match quirk::session_from_0_5(&buf) {
                    Ok(buf) => buf,
                    Err(e) => {
                        log::warn!("Corrupt session: {}. Continuing", e);
                        continue;
                    }
                },
                Err(e) if !path.exists() => {
                    log::trace!(
                        "Skipping session because session file does not exist ({})",
                        e
                    );
                    continue;
                }
                Err(e) => {
                    log::error!(
                        "Problem reading session: {}.  Skipping, but here be dragons.",
                        e
                    );
                    continue;
                }
            };

            // XXX Phone number possibly needs a + prefix or something like that.
            //     Maybe pull it through phonenumber for normalisation.
            let recipient = self.0.fetch_recipient(Some(addr.name()), Some(addr.name()));
            let recipient = if let Some(recipient) = recipient {
                recipient
            } else {
                // FIXME, we can create this recipient at this point
                log::warn!("No recipient for this session; leaving alone.");
                continue;
            };
            {
                use crate::schema::session_records::dsl::*;
                use diesel::prelude::*;
                let session_record = SessionRecord {
                    recipient_id: recipient.id,
                    device_id: addr.device_id() as i32,
                    record: buf,
                };
                let db = self.0.db.lock();
                diesel::insert_into(session_records)
                    .values(session_record)
                    .execute(&*db)
                    // XXX we should catch duplicate primary keys here.
                    .expect("inserting record into db");
            }

            // By now, the session is safely stored in the database, so we can remove the file.
            if let Err(e) = std::fs::remove_file(path) {
                log::debug!(
                    "Could not delete session {}, assuming non-existing: {}",
                    addr.to_string(),
                    e
                );
            }
        }
    }

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
