use super::*;
use libsignal_service::prelude::protocol::{self, Context};
use libsignal_service::provisioning::generate_registration_id;
use protocol::IdentityKeyPair;
use protocol::SignalProtocolError;
use std::path::Path;

pub struct ProtocolStore;

pub const DJB_TYPE: u8 = 0x05;

impl ProtocolStore {
    pub fn serialize_identity_key(identity_key_pair: IdentityKeyPair) -> Vec<u8> {
        // XXX move to quirk
        let mut identity_key = Vec::new();
        let public = identity_key_pair.public_key().serialize();
        assert_eq!(public.len(), 32 + 1);
        assert_eq!(public[0], DJB_TYPE);
        identity_key.extend(&public[1..]);

        let private = identity_key_pair.private_key().serialize();
        assert_eq!(private.len(), 32);
        identity_key.extend(private);

        identity_key
    }

    pub async fn new(
        store_enc: Option<&encryption::StorageEncryption>,
        path: &Path,
        regid: u32,
        pni_regid: u32,
        aci_identity_key_pair: IdentityKeyPair,
        pni_identity_key_pair: IdentityKeyPair,
    ) -> Result<Self, anyhow::Error> {
        // Identity
        let identity_path = path.join("storage").join("identity");

        let aci_identity_key = Self::serialize_identity_key(aci_identity_key_pair);
        let pni_identity_key = Self::serialize_identity_key(pni_identity_key_pair);

        // Encrypt regid if necessary and write to file
        utils::write_file_async_encrypted(
            identity_path.join("regid"),
            format!("{}", regid).into_bytes(),
            store_enc,
        )
        .await?;
        // Encrypt pni regid if necessary and write to file
        utils::write_file_async_encrypted(
            identity_path.join("pni_regid"),
            format!("{}", pni_regid).into_bytes(),
            store_enc,
        )
        .await?;

        // Encrypt identity key if necessary and write to file
        utils::write_file_async_encrypted(
            identity_path.join("identity_key"),
            aci_identity_key,
            store_enc,
        )
        .await?;

        // Encrypt PNI if necessary and write to file
        utils::write_file_async_encrypted(
            identity_path.join("pni_identity_key"),
            pni_identity_key,
            store_enc,
        )
        .await?;

        Ok(Self)
    }

    pub async fn open() -> Self {
        Self
    }
}

impl Storage {
    /// Returns a tuple of the next free signed pre-key ID and the next free pre-key ID
    pub async fn next_pre_key_ids(&self) -> (u32, u32) {
        use diesel::dsl::*;
        use diesel::prelude::*;

        let prekey_max: Option<i32> = {
            use crate::schema::prekeys::dsl::*;

            prekeys.select(max(id)).first(&mut *self.db()).expect("db")
        };
        let signed_prekey_max: Option<i32> = {
            use crate::schema::signed_prekeys::dsl::*;

            signed_prekeys
                .select(max(id))
                .first(&mut *self.db())
                .expect("db")
        };

        (
            (signed_prekey_max.unwrap_or(-1) + 1) as u32,
            (prekey_max.unwrap_or(-1) + 1) as u32,
        )
    }

    pub async fn delete_identity(&self, addr: &ProtocolAddress) -> Result<(), SignalProtocolError> {
        self.delete_identity_key(addr);
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl protocol::ProtocolStore for Storage {}

impl Storage {
    pub async fn get_local_pni_registration_id(
        &self,
        _: Context,
    ) -> Result<u32, SignalProtocolError> {
        let path = self.path.join("storage").join("identity").join("pni_regid");
        if !tokio::fs::try_exists(&path).await.expect("fs error") {
            log::info!("Generating PNI regid");
            let regid = generate_registration_id(&mut rand::thread_rng());

            // Encrypt regid if necessary and write to file
            self.write_file(&path, format!("{}", regid).into_bytes())
                .await
                .map_err(|e| {
                    SignalProtocolError::InvalidArgument(format!("Cannot write PNI regid {}", e))
                })?;
        }

        log::trace!("Reading PNI regid");
        let _lock = self.protocol_store.read().await;

        let regid = self.read_file(path).await.map_err(|e| {
            SignalProtocolError::InvalidArgument(format!("Cannot read PNI regid {}", e))
        })?;
        let regid = String::from_utf8(regid).map_err(|e| {
            SignalProtocolError::InvalidArgument(format!(
                "Convert PNI regid from bytes to string {}",
                e
            ))
        })?;
        let regid = regid.parse().map_err(|e| {
            SignalProtocolError::InvalidArgument(format!(
                "Convert PNI regid from string to number {}",
                e
            ))
        })?;

        Ok(regid)
    }
}

#[async_trait::async_trait(?Send)]
impl protocol::IdentityKeyStore for Storage {
    async fn get_identity_key_pair(
        &self,
        _: Context,
    ) -> Result<IdentityKeyPair, SignalProtocolError> {
        let identity_key_pair = self.aci_identity_key_pair.read().await;

        if let Some(identity_key_pair) = *identity_key_pair {
            Ok(identity_key_pair)
        } else {
            drop(identity_key_pair);

            let mut identity_key_pair = self.aci_identity_key_pair.write().await;

            let _lock = self.protocol_store.read().await;

            log::trace!("Reading own identity key pair");
            let path = self
                .path
                .join("storage")
                .join("identity")
                .join("identity_key");
            let key_pair = {
                use std::convert::TryFrom;
                let mut buf = self.read_file(path).await.map_err(|e| {
                    SignalProtocolError::InvalidArgument(format!(
                        "Cannot read own identity key {}",
                        e
                    ))
                })?;
                buf.insert(0, DJB_TYPE);
                let public = IdentityKey::decode(&buf[0..33])?;
                let private = PrivateKey::try_from(&buf[33..])?;
                IdentityKeyPair::new(public, private)
            };
            *identity_key_pair = Some(key_pair);
            Ok(identity_key_pair.unwrap())
        }
    }

    async fn get_local_registration_id(&self, _: Context) -> Result<u32, SignalProtocolError> {
        log::trace!("Reading regid");
        let _lock = self.protocol_store.read().await;

        let path = self.path.join("storage").join("identity").join("regid");
        let regid = self.read_file(path).await.map_err(|e| {
            SignalProtocolError::InvalidArgument(format!("Cannot read regid {}", e))
        })?;
        let regid = String::from_utf8(regid).map_err(|e| {
            SignalProtocolError::InvalidArgument(format!(
                "Convert regid from bytes to string {}",
                e
            ))
        })?;
        let regid = regid.parse().map_err(|e| {
            SignalProtocolError::InvalidArgument(format!(
                "Convert regid from string to number {}",
                e
            ))
        })?;

        Ok(regid)
    }

    async fn is_trusted_identity(
        &self,
        addr: &ProtocolAddress,
        key: &IdentityKey,
        // XXX
        _direction: Direction,
        _ctx: Context,
    ) -> Result<bool, SignalProtocolError> {
        if let Some(trusted_key) = self.fetch_identity_key(addr) {
            Ok(trusted_key == *key)
        } else {
            // Trust on first use
            Ok(true)
        }
    }

    /// Should return true when the older key, if present, is different from the new one.
    /// False otherwise.
    async fn save_identity(
        &mut self,
        addr: &ProtocolAddress,
        key: &IdentityKey,
        _: Context,
    ) -> Result<bool, SignalProtocolError> {
        Ok(self.store_identity_key(addr, key))
    }

    async fn get_identity(
        &self,
        addr: &ProtocolAddress,
        _: Context,
    ) -> Result<Option<IdentityKey>, SignalProtocolError> {
        Ok(self.fetch_identity_key(addr))
    }
}

#[async_trait::async_trait(?Send)]
impl protocol::PreKeyStore for Storage {
    async fn get_pre_key(
        &self,
        prekey_id: PreKeyId,
        _: Context,
    ) -> Result<PreKeyRecord, SignalProtocolError> {
        log::trace!("Loading prekey {}", prekey_id);
        use crate::schema::prekeys::dsl::*;
        use diesel::prelude::*;

        let prekey_record: Option<crate::store::orm::Prekey> = prekeys
            .filter(id.eq(u32::from(prekey_id) as i32))
            .first(&mut *self.db())
            .optional()
            .expect("db");
        if let Some(pkr) = prekey_record {
            Ok(PreKeyRecord::deserialize(&pkr.record)?)
        } else {
            Err(SignalProtocolError::InvalidPreKeyId)
        }
    }

    async fn save_pre_key(
        &mut self,
        prekey_id: PreKeyId,
        body: &PreKeyRecord,
        _: Context,
    ) -> Result<(), SignalProtocolError> {
        log::trace!("Storing prekey {}", prekey_id);
        use crate::schema::prekeys::dsl::*;
        use diesel::prelude::*;

        diesel::insert_into(prekeys)
            .values(crate::store::orm::Prekey {
                id: u32::from(prekey_id) as _,
                record: body.serialize()?,
            })
            .execute(&mut *self.db())
            .expect("db");

        Ok(())
    }

    async fn remove_pre_key(
        &mut self,
        prekey_id: PreKeyId,
        _: Context,
    ) -> Result<(), SignalProtocolError> {
        log::trace!("Removing prekey {}", prekey_id);
        use crate::schema::prekeys::dsl::*;
        use diesel::prelude::*;

        diesel::delete(prekeys)
            .filter(id.eq(u32::from(prekey_id) as i32))
            .execute(&mut *self.db())
            .expect("db");
        Ok(())
    }
}

impl Storage {
    // XXX Rewrite in terms of get_pre_key
    #[allow(dead_code)]
    async fn contains_pre_key(&self, prekey_id: u32) -> bool {
        log::trace!("Checking for prekey {}", prekey_id);
        use crate::schema::prekeys::dsl::*;
        use diesel::prelude::*;

        let prekey_record: Option<crate::store::orm::Prekey> = prekeys
            .filter(id.eq(prekey_id as i32))
            .first(&mut *self.db())
            .optional()
            .expect("db");
        prekey_record.is_some()
    }
}

#[async_trait::async_trait(?Send)]
impl protocol::SessionStore for Storage {
    async fn load_session(
        &self,
        addr: &ProtocolAddress,
        _: Context,
    ) -> Result<Option<SessionRecord>, SignalProtocolError> {
        log::trace!("Loading session for {}", addr);
        use crate::schema::session_records::dsl::*;
        use diesel::prelude::*;

        let session_record: Option<crate::store::orm::SessionRecord> = session_records
            .filter(
                address
                    .eq(addr.name())
                    .and(device_id.eq(u32::from(addr.device_id()) as i32)),
            )
            .first(&mut *self.db())
            .optional()
            .expect("db");
        if let Some(session_record) = session_record {
            Ok(Some(SessionRecord::deserialize(&session_record.record)?))
        } else {
            Ok(None)
        }
    }

    async fn store_session(
        &mut self,
        addr: &ProtocolAddress,
        session: &protocol::SessionRecord,
        context: Context,
    ) -> Result<(), SignalProtocolError> {
        log::trace!("Storing session for {}", addr);
        use crate::schema::session_records::dsl::*;
        use diesel::prelude::*;

        if self.contains_session(addr, context).await? {
            diesel::update(session_records)
                .filter(
                    address
                        .eq(addr.name())
                        .and(device_id.eq(u32::from(addr.device_id()) as i32)),
                )
                .set(record.eq(session.serialize()?))
                .execute(&mut *self.db())
                .expect("updated session");
        } else {
            diesel::insert_into(session_records)
                .values((
                    address.eq(addr.name()),
                    device_id.eq(u32::from(addr.device_id()) as i32),
                    record.eq(session.serialize()?),
                ))
                .execute(&mut *self.db())
                .expect("updated session");
        }

        Ok(())
    }
}

impl Storage {
    #[allow(dead_code)]
    /// Check whether session exists.
    ///
    /// This does *not* lock the protocol store.  If a transactional check is required, use the
    /// lock from outside.
    async fn contains_session(
        &self,
        addr: &ProtocolAddress,
        _: Context,
    ) -> Result<bool, SignalProtocolError> {
        use crate::schema::session_records::dsl::*;
        use diesel::dsl::*;
        use diesel::prelude::*;

        let count: i64 = session_records
            .select(count_star())
            .filter(
                address
                    .eq(addr.name())
                    .and(device_id.eq(u32::from(addr.device_id()) as i32)),
            )
            .first(&mut *self.db())
            .expect("db");
        Ok(count != 0)
    }
}

// BEGIN identity key block
impl Storage {
    /// Fetches the identity matching `addr` from the database
    ///
    /// Does not lock the protocol storage.
    fn fetch_identity_key(&self, addr: &ProtocolAddress) -> Option<IdentityKey> {
        use crate::schema::identity_records::dsl::*;
        let addr = addr.name();
        let found: orm::IdentityRecord = identity_records
            .filter(address.eq(addr))
            .first(&mut *self.db())
            .optional()
            .expect("db")?;

        Some(IdentityKey::decode(&found.record).expect("only valid identity keys in db"))
    }

    /// Removes the identity matching `addr` from the database
    ///
    /// Does not lock the protocol storage.
    pub fn delete_identity_key(&self, addr: &ProtocolAddress) -> bool {
        use crate::schema::identity_records::dsl::*;
        let addr = addr.name();
        let amount = diesel::delete(identity_records)
            .filter(address.eq(addr))
            .execute(&mut *self.db())
            .expect("db");

        amount == 1
    }

    /// (Over)writes the identity key for a given address.
    ///
    /// Returns whether the identity key has been altered.
    fn store_identity_key(&self, addr: &ProtocolAddress, key: &IdentityKey) -> bool {
        use crate::schema::identity_records::dsl::*;
        let previous = self.fetch_identity_key(addr);

        let ret = previous.as_ref() == Some(key);

        if previous.is_some() {
            diesel::update(identity_records)
                .filter(address.eq(addr.name()))
                .set(record.eq(key.serialize().to_vec()))
                .execute(&mut *self.db())
                .expect("db");
        } else {
            diesel::insert_into(identity_records)
                .values((address.eq(addr.name()), record.eq(key.serialize().to_vec())))
                .execute(&mut *self.db())
                .expect("db");
        }

        ret
    }
}
// END identity key

#[async_trait::async_trait(?Send)]
impl protocol::SessionStoreExt for Storage {
    async fn get_sub_device_sessions(
        &self,
        addr: &ServiceAddress,
    ) -> Result<Vec<u32>, SignalProtocolError> {
        log::trace!("Looking for sub_device sessions for {:?}", addr);
        use crate::schema::session_records::dsl::*;

        let records: Vec<i32> = session_records
            .select(device_id)
            .filter(
                address
                    .eq(addr.uuid.to_string())
                    .and(device_id.ne(libsignal_service::push_service::DEFAULT_DEVICE_ID as i32)),
            )
            .load(&mut *self.db())
            .expect("db");
        Ok(records.into_iter().map(|x| x as u32).collect())
    }

    async fn delete_session(&self, addr: &ProtocolAddress) -> Result<(), SignalProtocolError> {
        use crate::schema::session_records::dsl::*;

        let num = diesel::delete(session_records)
            .filter(
                address
                    .eq(addr.name())
                    .and(device_id.eq(u32::from(addr.device_id()) as i32)),
            )
            .execute(&mut *self.db())
            .expect("db");

        if num != 1 {
            log::debug!(
                "Could not delete session {}, assuming non-existing.",
                addr.to_string(),
            );
            Err(SignalProtocolError::SessionNotFound(addr.clone()))
        } else {
            Ok(())
        }
    }

    async fn delete_all_sessions(
        &self,
        addr: &ServiceAddress,
    ) -> Result<usize, SignalProtocolError> {
        log::warn!("Deleting all sessions for {:?}", addr);
        use crate::schema::session_records::dsl::*;

        let num = diesel::delete(session_records)
            .filter(address.eq(addr.uuid.to_string()))
            .execute(&mut *self.db())
            .expect("db");

        Ok(num)
    }
}

#[async_trait::async_trait(?Send)]
impl protocol::SignedPreKeyStore for Storage {
    async fn get_signed_pre_key(
        &self,
        signed_prekey_id: SignedPreKeyId,
        _: Context,
    ) -> Result<SignedPreKeyRecord, SignalProtocolError> {
        log::trace!("Loading signed prekey {}", signed_prekey_id);
        use crate::schema::signed_prekeys::dsl::*;
        use diesel::prelude::*;

        let prekey_record: Option<crate::store::orm::SignedPrekey> = signed_prekeys
            .filter(id.eq(u32::from(signed_prekey_id) as i32))
            .first(&mut *self.db())
            .optional()
            .expect("db");
        if let Some(pkr) = prekey_record {
            Ok(SignedPreKeyRecord::deserialize(&pkr.record)?)
        } else {
            Err(SignalProtocolError::InvalidSignedPreKeyId)
        }
    }

    async fn save_signed_pre_key(
        &mut self,
        signed_prekey_id: SignedPreKeyId,
        body: &SignedPreKeyRecord,
        _: Context,
    ) -> Result<(), SignalProtocolError> {
        log::trace!("Storing prekey {}", signed_prekey_id);
        use crate::schema::signed_prekeys::dsl::*;
        use diesel::prelude::*;

        // Insert or replace?
        diesel::insert_into(signed_prekeys)
            .values(crate::store::orm::SignedPrekey {
                id: u32::from(signed_prekey_id) as _,
                record: body.serialize()?,
            })
            .execute(&mut *self.db())
            .expect("db");

        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl SenderKeyStore for Storage {
    async fn store_sender_key(
        &mut self,
        addr: &ProtocolAddress,
        distr_id: Uuid,
        record: &SenderKeyRecord,
        _: Context,
    ) -> Result<(), SignalProtocolError> {
        log::trace!("Storing sender key {} {}", addr, distr_id);

        let to_insert = orm::SenderKeyRecord {
            address: addr.name().to_owned(),
            device: u32::from(addr.device_id()) as i32,
            distribution_id: distr_id.to_string(),
            record: record.serialize()?,
            created_at: Utc::now().naive_utc(),
        };

        {
            use crate::schema::sender_key_records::dsl::*;
            diesel::insert_into(sender_key_records)
                .values(to_insert)
                .execute(&mut *self.db())
                .expect("db");
        }
        Ok(())
    }
    async fn load_sender_key(
        &mut self,
        addr: &ProtocolAddress,
        distr_id: Uuid,
        _: Context,
    ) -> Result<Option<SenderKeyRecord>, SignalProtocolError> {
        log::trace!("Loading sender key {} {}", addr, distr_id);

        let found: Option<orm::SenderKeyRecord> = {
            use crate::schema::sender_key_records::dsl::*;
            sender_key_records
                .filter(
                    address
                        .eq(addr.name())
                        .and(device.eq(u32::from(addr.device_id()) as i32))
                        .and(distribution_id.eq(distr_id.to_string())),
                )
                .first(&mut *self.db())
                .optional()
                .expect("db")
        };

        match found {
            Some(x) => Ok(Some(SenderKeyRecord::deserialize(&x.record)?)),
            None => Ok(None),
        }
    }
}

impl Storage {
    #[allow(dead_code)]
    async fn remove_signed_pre_key(
        &self,
        signed_prekey_id: u32,
    ) -> Result<(), SignalProtocolError> {
        log::trace!("Removing signed prekey {}", signed_prekey_id);
        use crate::schema::signed_prekeys::dsl::*;
        use diesel::prelude::*;

        diesel::delete(signed_prekeys)
            .filter(id.eq(signed_prekey_id as i32))
            .execute(&mut *self.db())
            .expect("db");
        Ok(())
    }

    // XXX rewrite in terms of get_signed_pre_key
    #[allow(dead_code)]
    async fn contains_signed_pre_key(&self, signed_prekey_id: u32) -> bool {
        log::trace!("Checking for signed prekey {}", signed_prekey_id);
        use crate::schema::signed_prekeys::dsl::*;
        use diesel::prelude::*;

        let signed_prekey_record: Option<crate::store::orm::SignedPrekey> = signed_prekeys
            .filter(id.eq(signed_prekey_id as i32))
            .first(&mut *self.db())
            .optional()
            .expect("db");
        signed_prekey_record.is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use libsignal_service::{prelude::protocol::*, ServiceAddress};
    use rstest::rstest;

    use crate::config::SignalConfig;

    async fn create_example_storage(
        storage_password: Option<&str>,
    ) -> Result<(super::Storage, super::StorageLocation<tempfile::TempDir>), anyhow::Error> {
        use rand::distributions::Alphanumeric;
        use rand::{Rng, RngCore};

        env_logger::try_init().ok();

        let location = super::temp();
        let rng = rand::thread_rng();

        // Signaling password for REST API
        let password: String = rng.sample_iter(&Alphanumeric).take(24).collect();

        // Signaling key that decrypts the incoming Signal messages
        let mut rng = rand::thread_rng();
        let mut signaling_key = [0u8; 52];
        rng.fill_bytes(&mut signaling_key);
        let signaling_key = signaling_key;

        // Registration ID
        let regid = 12345;
        let pni_regid = 12345;

        let storage = super::Storage::new(
            Arc::new(SignalConfig::default()),
            &location,
            storage_password,
            regid,
            pni_regid,
            &password,
            signaling_key,
            None,
            None,
        )
        .await?;

        Ok((storage, location))
    }

    fn create_random_protocol_address() -> (ServiceAddress, ProtocolAddress) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let user_id = uuid::Uuid::new_v4();
        let device_id = rng.gen_range(2, 20);

        let svc = ServiceAddress::from(user_id);
        let prot = ProtocolAddress::new(user_id.to_string(), DeviceId::from(device_id));
        (svc, prot)
    }

    fn create_random_identity_key() -> IdentityKey {
        let mut rng = rand::thread_rng();

        let key_pair = IdentityKeyPair::generate(&mut rng);

        *key_pair.identity_key()
    }

    fn create_random_prekey() -> PreKeyRecord {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let key_pair = KeyPair::generate(&mut rng);
        let id: u32 = rng.gen();

        PreKeyRecord::new(PreKeyId::from(id), &key_pair)
    }

    fn create_random_signed_prekey() -> SignedPreKeyRecord {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let key_pair = KeyPair::generate(&mut rng);
        let id: u32 = rng.gen();
        let timestamp: u64 = rng.gen();
        let signature = vec![0; 3];

        SignedPreKeyRecord::new(SignedPreKeyId::from(id), timestamp, &key_pair, &signature)
    }

    /// XXX Right now, this functions seems a bit unnecessary, but we will change the creation of a
    /// storage and it might be necessary to check the own identity_key_pair in the protocol store.
    #[rstest(password, case(Some("some password")), case(None))]
    #[tokio::test]
    async fn own_identity_key_pair(password: Option<&str>) {
        env_logger::try_init().ok();

        // create a new storage
        let (storage, _tempdir) = create_example_storage(password).await.unwrap();

        // Copy the identity key pair
        let id_key1 = storage.get_identity_key_pair(None).await.unwrap();

        // Get access to the protocol store
        // XXX IdentityKeyPair does not implement the std::fmt::Debug trait *arg*
        //assert_eq!(id_key1.unwrap(), store.get_identity_key_pair(None).await.unwrap());
        assert_eq!(
            id_key1.serialize(),
            storage
                .get_identity_key_pair(None)
                .await
                .unwrap()
                .serialize()
        );
    }

    /// XXX Right now, this functions seems a bit unnecessary, but we will change the creation of a
    /// storage and it might be necessary to check the regid in the protocol store.
    #[rstest(password, case(Some("some password")), case(None))]
    #[tokio::test]
    async fn own_regid(password: Option<&str>) {
        env_logger::try_init().ok();

        // create a new storage
        let (storage, _tempdir) = create_example_storage(password).await.unwrap();

        // Copy the regid
        let regid_1 = storage.get_local_registration_id(None).await.unwrap();

        // Get access to the protocol store
        assert_eq!(
            regid_1,
            storage.get_local_registration_id(None).await.unwrap()
        );
    }

    #[rstest(password, case(Some("some password")), case(None))]
    #[tokio::test]
    async fn save_retrieve_identity_key(password: Option<&str>) {
        env_logger::try_init().ok();

        // Create a new storage
        let (mut storage, _tempdir) = create_example_storage(password).await.unwrap();

        // We need two identity keys and two addresses
        let (_svc1, addr1) = create_random_protocol_address();
        let (_svc2, addr2) = create_random_protocol_address();
        let key1 = create_random_identity_key();
        let key2 = create_random_identity_key();

        // In the beginning, the storage should be emtpy and return an error
        // XXX Doesn't implement equality *arg*
        assert_eq!(storage.get_identity(&addr1, None).await.unwrap(), None);
        assert_eq!(storage.get_identity(&addr2, None).await.unwrap(), None);

        // We store both keys and should get false because there wasn't a key with that address
        // yet
        assert!(!storage.save_identity(&addr1, &key1, None).await.unwrap());
        assert!(!storage.save_identity(&addr2, &key2, None).await.unwrap());

        // Now, we should get both keys
        assert_eq!(
            storage.get_identity(&addr1, None).await.unwrap(),
            Some(key1)
        );
        assert_eq!(
            storage.get_identity(&addr2, None).await.unwrap(),
            Some(key2)
        );

        // After removing key2, it shouldn't be there
        storage.delete_identity(&addr2).await.unwrap();
        // XXX Doesn't implement equality *arg*
        assert_eq!(storage.get_identity(&addr2, None).await.unwrap(), None);

        // We can now overwrite key1 with key1 and should get true returned
        assert!(storage.save_identity(&addr1, &key1, None).await.unwrap());

        // We can now overwrite key1 with key2 and should get false returned
        assert!(!storage.save_identity(&addr1, &key2, None).await.unwrap());
    }

    // Direction does not matter yet
    #[rstest(password, case(Some("some password")), case(None))]
    #[tokio::test]
    async fn is_trusted_identity(password: Option<&str>) {
        env_logger::try_init().ok();

        // Create a new storage
        let (mut storage, _tempdir) = create_example_storage(password).await.unwrap();

        // We need two identity keys and two addresses
        let (_, addr1) = create_random_protocol_address();
        let key1 = create_random_identity_key();
        let key2 = create_random_identity_key();

        // Test trust on first use
        assert!(storage
            .is_trusted_identity(&addr1, &key1, Direction::Receiving, None)
            .await
            .unwrap());

        // Test inserted key
        storage.save_identity(&addr1, &key1, None).await.unwrap();
        assert!(storage
            .is_trusted_identity(&addr1, &key1, Direction::Receiving, None)
            .await
            .unwrap());

        // Test wrong key
        assert!(!storage
            .is_trusted_identity(&addr1, &key2, Direction::Receiving, None)
            .await
            .unwrap());
    }

    #[rstest(password, case(Some("some password")), case(None))]
    #[tokio::test]
    async fn save_retrieve_prekey(password: Option<&str>) {
        env_logger::try_init().ok();

        // Create a new storage
        let (mut storage, _tempdir) = create_example_storage(password).await.unwrap();

        // We need two identity keys and two addresses
        let id1 = 0u32;
        let id2 = 1u32;
        let key1 = create_random_prekey();
        let key2 = create_random_prekey();

        // In the beginning, the storage should be emtpy and return an error
        // XXX Doesn't implement equality *arg*
        assert_eq!(
            storage
                .get_pre_key(PreKeyId::from(id1), None)
                .await
                .unwrap_err()
                .to_string(),
            SignalProtocolError::InvalidPreKeyId.to_string()
        );

        // Storing both keys and testing retrieval
        storage
            .save_pre_key(PreKeyId::from(id1), &key1, None)
            .await
            .unwrap();
        storage
            .save_pre_key(PreKeyId::from(id2), &key2, None)
            .await
            .unwrap();

        // Now, we should get both keys
        assert_eq!(
            storage
                .get_pre_key(PreKeyId::from(id1), None)
                .await
                .unwrap()
                .serialize()
                .unwrap(),
            key1.serialize().unwrap()
        );
        assert_eq!(
            storage
                .get_pre_key(PreKeyId::from(id2), None)
                .await
                .unwrap()
                .serialize()
                .unwrap(),
            key2.serialize().unwrap()
        );

        // After removing key2, it shouldn't be there
        storage
            .remove_pre_key(PreKeyId::from(id2), None)
            .await
            .unwrap();
        // XXX Doesn't implement equality *arg*
        assert_eq!(
            storage
                .get_pre_key(PreKeyId::from(id2), None)
                .await
                .unwrap_err()
                .to_string(),
            SignalProtocolError::InvalidPreKeyId.to_string()
        );

        // Let's check whether we can overwrite a key
        storage
            .save_pre_key(PreKeyId::from(id1), &key2, None)
            .await
            .unwrap();
    }

    #[rstest(password, case(Some("some password")), case(None))]
    #[tokio::test]
    async fn save_retrieve_signed_prekey(password: Option<&str>) {
        env_logger::try_init().ok();

        // Create a new storage
        let (mut storage, _tempdir) = create_example_storage(password).await.unwrap();

        // We need two identity keys and two addresses
        let id1 = 0u32;
        let id2 = 1u32;
        let key1 = create_random_signed_prekey();
        let key2 = create_random_signed_prekey();

        // In the beginning, the storage should be emtpy and return an error
        // XXX Doesn't implement equality *arg*
        assert_eq!(
            storage
                .get_signed_pre_key(SignedPreKeyId::from(id1), None)
                .await
                .unwrap_err()
                .to_string(),
            SignalProtocolError::InvalidSignedPreKeyId.to_string()
        );

        // Storing both keys and testing retrieval
        storage
            .save_signed_pre_key(SignedPreKeyId::from(id1), &key1, None)
            .await
            .unwrap();
        storage
            .save_signed_pre_key(SignedPreKeyId::from(id2), &key2, None)
            .await
            .unwrap();

        // Now, we should get both keys
        assert_eq!(
            storage
                .get_signed_pre_key(SignedPreKeyId::from(id1), None)
                .await
                .unwrap()
                .serialize()
                .unwrap(),
            key1.serialize().unwrap()
        );
        assert_eq!(
            storage
                .get_signed_pre_key(SignedPreKeyId::from(id2), None)
                .await
                .unwrap()
                .serialize()
                .unwrap(),
            key2.serialize().unwrap()
        );

        // Let's check whether we can overwrite a key
        storage
            .save_signed_pre_key(SignedPreKeyId::from(id1), &key2, None)
            .await
            .unwrap();
    }

    #[rstest(password, case(Some("some password")), case(None))]
    #[tokio::test]
    async fn save_retrieve_session(password: Option<&str>) {
        env_logger::try_init().ok();

        // Create a new storage
        let (mut storage, _tempdir) = create_example_storage(password).await.unwrap();

        // Collection of some addresses and sessions
        let (_svc1, addr1) = create_random_protocol_address();
        let (_svc2, addr2) = create_random_protocol_address();
        let (svc3, addr3) = create_random_protocol_address();
        let addr4 = ProtocolAddress::new(
            addr3.name().to_string(),
            DeviceId::from(u32::from(addr3.device_id()) + 1),
        );
        let session1 = SessionRecord::new_fresh();
        let session2 = SessionRecord::new_fresh();
        let session3 = SessionRecord::new_fresh();
        let session4 = SessionRecord::new_fresh();

        // In the beginning, the storage should be emtpy and return an error
        assert!(storage.load_session(&addr1, None).await.unwrap().is_none());
        assert!(storage.load_session(&addr2, None).await.unwrap().is_none());

        // Store all four sessions: three different names, one name with two different device ids.
        storage
            .store_session(&addr1, &session1, None)
            .await
            .unwrap();
        storage
            .store_session(&addr2, &session2, None)
            .await
            .unwrap();
        storage
            .store_session(&addr3, &session3, None)
            .await
            .unwrap();
        storage
            .store_session(&addr4, &session4, None)
            .await
            .unwrap();

        // Now, we should get the sessions to the first two addresses
        assert_eq!(
            storage
                .load_session(&addr1, None)
                .await
                .unwrap()
                .unwrap()
                .serialize()
                .unwrap(),
            session1.serialize().unwrap()
        );
        assert_eq!(
            storage
                .load_session(&addr2, None)
                .await
                .unwrap()
                .unwrap()
                .serialize()
                .unwrap(),
            session2.serialize().unwrap()
        );

        // Let's check whether we can overwrite a key
        storage
            .store_session(&addr1, &session2, None)
            .await
            .expect("Overwrite session");

        // Get all device ids for the same address
        let mut ids = storage.get_sub_device_sessions(&svc3).await.unwrap();
        ids.sort_unstable();
        assert_eq!(
            DeviceId::from(ids[0]),
            std::cmp::min(addr3.device_id(), addr4.device_id())
        );
        assert_eq!(
            DeviceId::from(ids[1]),
            std::cmp::max(addr3.device_id(), addr4.device_id())
        );

        // If we call delete all sessions, all sessions of one person/address should be removed
        assert_eq!(storage.delete_all_sessions(&svc3).await.unwrap(), 2);
        assert!(storage.load_session(&addr3, None).await.unwrap().is_none());
        assert!(storage.load_session(&addr4, None).await.unwrap().is_none());

        // If we delete the first two sessions, they shouldn't be in the store anymore
        SessionStoreExt::delete_session(&storage, &addr1)
            .await
            .unwrap();
        SessionStoreExt::delete_session(&storage, &addr2)
            .await
            .unwrap();
        assert!(storage.load_session(&addr1, None).await.unwrap().is_none());
        assert!(storage.load_session(&addr2, None).await.unwrap().is_none());
    }

    #[rstest(password, case(Some("some password")), case(None))]
    #[tokio::test]
    async fn get_next_pre_key_ids(password: Option<&str>) {
        env_logger::try_init().ok();

        // Create a new storage
        let (mut storage, _tempdir) = create_example_storage(password).await.unwrap();

        // Create two pre keys and one signed pre key
        let key1 = create_random_prekey();
        let key2 = create_random_prekey();
        let key3 = create_random_signed_prekey();

        // In the beginning zero should be returned
        assert_eq!(storage.next_pre_key_ids().await, (0, 0));

        // Now, we add our keys
        storage
            .save_pre_key(PreKeyId::from(0), &key1, None)
            .await
            .unwrap();
        storage
            .save_pre_key(PreKeyId::from(1), &key2, None)
            .await
            .unwrap();
        storage
            .save_signed_pre_key(SignedPreKeyId::from(0), &key3, None)
            .await
            .unwrap();

        // Adapt to keys in the storage
        assert_eq!(storage.next_pre_key_ids().await, (1, 2));
    }
}
