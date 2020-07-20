use std::io::{self, Write};
use std::path::Path;

use libsignal_protocol::stores::SerializedSession;
use libsignal_protocol::stores::{IdentityKeyStore, PreKeyStore, SessionStore, SignedPreKeyStore};
use libsignal_protocol::InternalError;
use libsignal_protocol::{Address, Buffer};

mod quirk;

use super::*;

pub struct ProtocolStore {
    identity_key: Vec<u8>,
    regid: u32,
}

impl ProtocolStore {
    pub async fn open_with_key(keys: [u8; 16 + 20], path: &Path) -> Result<Self, failure::Error> {
        // Identity
        let identity_path = path.join("storage").join("identity");

        let regid = load_file(keys, identity_path.join("regid")).await?;
        let regid = String::from_utf8(regid)?;
        let regid = regid.parse()?;
        let identity_key = load_file(keys, identity_path.join("identity_key")).await?;

        Ok(Self {
            identity_key,
            regid,
        })
    }
}

impl Storage {
    // XXX: this is made to be Go-compatible: only accept addr's that start with + (phone number).
    // Signal is moving away from this.  Uuid-based paths will work perfectly, but will *not* be
    // backwards compatible with 0.5.
    fn session_path(&self, addr: &Address) -> Option<PathBuf> {
        let addr_str = addr.as_str().unwrap();
        let recipient_id = if addr_str.starts_with('+') {
            // strip the prefix + from e164, as is done in Go (cfr. the `func recID`).
            &addr_str[1..]
        } else {
            return None;
            // addr_str
        };

        Some(
            crate::store::default_location()
                .unwrap()
                .join("storage")
                .join("sessions")
                .join(format!("{}_{}", recipient_id, addr.device_id())),
        )
    }
}

impl IdentityKeyStore for Storage {
    fn identity_key_pair(&self) -> Result<(Buffer, Buffer), InternalError> {
        log::trace!("identity_key_pair");
        let protocol_store = self.protocol_store.lock().expect("mutex");
        // (public, private)
        Ok((
            Buffer::from(&protocol_store.identity_key[..32]),
            Buffer::from(&protocol_store.identity_key[32..]),
        ))
    }
    fn local_registration_id(&self) -> Result<u32, InternalError> {
        Ok(self.protocol_store.lock().expect("mutex").regid)
    }
    fn is_trusted_identity(&self, _: Address, _: &[u8]) -> Result<bool, InternalError> {
        todo!("is_trusted_identity")
    }
    fn save_identity(&self, _: Address, _: &[u8]) -> Result<(), InternalError> {
        todo!("save_identity")
    }
}

impl PreKeyStore for Storage {
    fn load(&self, _: u32, _: &mut dyn Write) -> Result<(), io::Error> {
        todo!("load")
    }
    fn store(&self, _: u32, _: &[u8]) -> Result<(), InternalError> {
        todo!("store")
    }
    fn contains(&self, _: u32) -> bool {
        todo!("contains")
    }
    fn remove(&self, _: u32) -> Result<(), InternalError> {
        todo!("remove")
    }
}

impl SessionStore for Storage {
    fn load_session(&self, addr: Address) -> Result<Option<SerializedSession>, InternalError> {
        let path = if let Some(path) = self.session_path(&addr) {
            path
        } else {
            return Ok(None);
        };

        log::trace!("Loading session for {:?} from {:?}", addr, path);

        let buf = load_file_sync(self.keys.unwrap(), path).expect("readable file");

        Ok(Some(SerializedSession {
            session: Buffer::from(&quirk::from_0_5(&buf)? as &[u8]),
            extra_data: None,
        }))
    }

    fn get_sub_device_sessions(&self, addr: &[u8]) -> Result<std::vec::Vec<i32>, InternalError> {
        let session_dir = crate::store::default_location()
            .unwrap()
            .join("storage")
            .join("sessions");

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
                    if name[addr.len()] != '_' as u8 {
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
            .collect();

        Ok(ids)
    }

    fn contains_session(&self, addr: Address) -> Result<bool, InternalError> {
        let path = self.session_path(&addr);
        if let Some(path) = path {
            Ok(path.is_file())
        } else {
            Ok(false)
        }
    }

    fn store_session(
        &self,
        addr: Address,
        session: libsignal_protocol::stores::SerializedSession,
    ) -> Result<(), InternalError> {
        let path = self.session_path(&addr).expect("path for session FIXME");

        log::trace!("Storing session for {:?} at {:?}", addr, path);

        let quirked = quirk::to_0_5(&session.session.as_slice())?;
        write_file_sync(self.keys.unwrap(), path, &quirked).unwrap();
        Ok(())
    }

    fn delete_session(&self, addr: Address) -> Result<(), InternalError> {
        let _path = self.session_path(&addr);
        todo!("delete_session")
    }

    fn delete_all_sessions(&self, _: &[u8]) -> Result<usize, InternalError> {
        todo!("delete_all_sessions")
    }
}

impl SignedPreKeyStore for Storage {
    fn load(&self, _: u32, _: &mut dyn Write) -> Result<(), io::Error> {
        todo!("load")
    }
    fn store(&self, _: u32, _: &[u8]) -> Result<(), InternalError> {
        todo!("store")
    }
    fn contains(&self, _: u32) -> bool {
        todo!("contains")
    }
    fn remove(&self, _: u32) -> Result<(), InternalError> {
        todo!("remove")
    }
}
