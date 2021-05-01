use actix::prelude::*;

use libsignal_service::profile_name::ProfileName;
use libsignal_service::push_service::DeviceCapabilities;
use rand::Rng;
use zkgroup::profiles::ProfileKey;

use crate::store::TrustLevel;

use super::*;

/// Generate and upload a profile for the self recipient.
#[derive(Message)]
#[rtype(result = "()")]
pub struct GenerateEmptyProfileIfNeeded;

/// Synchronize multi-device profile information.
#[derive(Message)]
#[rtype(result = "()")]
pub struct MultideviceSyncProfile;

/// Synchronize profile attributes.
#[derive(Message)]
#[rtype(result = "()")]
pub struct RefreshProfileAttributes;

impl Handler<MultideviceSyncProfile> for ClientActor {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, _: MultideviceSyncProfile, _ctx: &mut Self::Context) -> Self::Result {
        let storage = self.storage.clone().unwrap();
        let local_addr = self.local_addr.clone().unwrap();

        let mut sender = MessageSender::new(
            self.authenticated_service(),
            self.cipher.clone().unwrap(),
            rand::thread_rng(),
            storage.clone(),
            storage.clone(),
            local_addr.clone(),
            DEFAULT_DEVICE_ID,
        );
        let config = self.config.clone();

        Box::pin(async move {
            let self_recipient = storage
                .fetch_self_recipient(&config)
                .expect("self recipient should be set by now");

            use libsignal_service::sender::ContactDetails;

            let contacts = std::iter::once(ContactDetails {
                number: self_recipient.e164.clone(),
                uuid: self_recipient.uuid.clone(),
                name: self_recipient.profile_joined_name.clone(),
                profile_key: self_recipient.profile_key,
                // XXX other profile stuff
                ..Default::default()
            });

            if let Err(e) = sender
                .send_contact_details(&local_addr, None, contacts, false, false)
                .await
            {
                log::warn!("Could not sync profile key: {}", e);
            }
        })
    }
}

impl Handler<GenerateEmptyProfileIfNeeded> for ClientActor {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, _: GenerateEmptyProfileIfNeeded, ctx: &mut Self::Context) -> Self::Result {
        let storage = self.storage.clone().unwrap();
        let service = self.authenticated_service();
        let client = ctx.address();
        let config = self.config.clone();
        let uuid = config.get_uuid_clone();
        let uuid = uuid::Uuid::parse_str(&uuid).expect("valid uuid at this point");

        Box::pin(async move {
            let self_recipient = storage
                .fetch_self_recipient(&config)
                .expect("self recipient should be set by now");
            if let Some(key) = self_recipient.profile_key {
                log::trace!(
                    "Profile key is already set ({} bytes); not overwriting",
                    key.len()
                );
                return;
            }

            log::info!("Generating profile key");
            let profile_key = ProfileKey::generate(rand::thread_rng().gen());
            let mut am = AccountManager::new(service, Some(profile_key.get_bytes()));
            am.upload_versioned_profile_without_avatar(uuid, ProfileName::empty(), None, None)
                .await
                .expect("upload profile");

            // Now also set the database
            storage.update_profile_key(
                None,
                Some(&uuid.to_string()),
                &profile_key.get_bytes(),
                TrustLevel::Certain,
            );

            client.send(RefreshProfileAttributes).await.unwrap();
            client.send(MultideviceSyncProfile).await.unwrap();
        })
    }
}

impl Handler<RefreshProfileAttributes> for ClientActor {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, _: RefreshProfileAttributes, _ctx: &mut Self::Context) -> Self::Result {
        let storage = self.storage.clone().unwrap();
        let protocol_store = storage.protocol_store.clone();
        let service = self.authenticated_service();
        let config = self.config.clone();

        Box::pin(async move {
            let registration_id = protocol_store.read().await.regid;
            let self_recipient = storage
                .fetch_self_recipient(&config)
                .expect("self set by now");

            let mut am = AccountManager::new(service, self_recipient.profile_key());
            am.set_account_attributes(
                None,            // signaling key
                registration_id, // regid
                false,           // voice
                false,           // video
                true,            // fetches_messages
                None,            // pin
                None,            // reg lock
                None,            // unidentified_access_key
                false,           //unresticted UA
                true,            // discoverable by phone
                DeviceCapabilities {
                    uuid: true,
                    gv2: true,
                    storage: false,
                    gv1_migration: true,
                },
            )
            .await
            .expect("upload profile");
            log::info!("Profile attributes refreshed");
        })
    }
}
