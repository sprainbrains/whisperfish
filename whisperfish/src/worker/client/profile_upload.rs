use super::{profile::ProfileFetched, *};
use crate::store::TrustLevel;
use actix::prelude::*;
use libsignal_service::profile_name::ProfileName;
use rand::Rng;
use zkgroup::profiles::ProfileKey;

/// Refresh the profile for the self recipient.
#[derive(Message)]
#[rtype(result = "()")]
pub struct RefreshOwnProfile {
    pub force: bool,
}

/// Generate and upload a profile for the self recipient.
#[derive(Message)]
#[rtype(result = "()")]
pub struct UploadProfile;

/// Provide new profile details and upload the profile for the self recipient.
#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateProfile {
    pub given_name: String,
    pub family_name: String,
    pub about: String,
    pub emoji: String,
}

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
    fn handle(&mut self, _: MultideviceSyncProfile, ctx: &mut Self::Context) -> Self::Result {
        let storage = self.storage.clone().unwrap();
        let local_addr = self.local_addr.unwrap();

        // If not yet connected, retry in 60 seconds
        if self.ws.is_none() {
            ctx.notify_later(MultideviceSyncProfile, Duration::from_secs(60));
            return Box::pin(async move {});
        }

        let sender = self.message_sender();
        let addr = ctx.address();

        Box::pin(async move {
            let mut sender = match sender.await {
                Ok(s) => s,
                Err(_e) => {
                    actix::clock::sleep(Duration::from_secs(60)).await;
                    addr.send(MultideviceSyncProfile)
                        .await
                        .expect("actor alive");
                    return;
                }
            };
            let self_recipient = storage
                .fetch_self_recipient()
                .expect("self recipient should be set by now");

            use libsignal_service::sender::ContactDetails;

            let contacts = std::iter::once(ContactDetails {
                number: self_recipient.e164.as_ref().map(PhoneNumber::to_string),
                uuid: self_recipient.uuid.as_ref().map(Uuid::to_string),
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

impl Handler<RefreshOwnProfile> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(
        &mut self,
        RefreshOwnProfile { force }: RefreshOwnProfile,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let storage = self.storage.clone().unwrap();
        let mut service = self.authenticated_service();
        let client = ctx.address();
        let config = self.config.clone();
        let uuid = config.get_uuid().expect("valid uuid at this point");

        Box::pin(
            async move {
                let self_recipient = storage
                    .fetch_self_recipient()
                    .expect("self recipient should be set by now");
                let profile_key = self_recipient.profile_key.map(|bytes| {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&bytes);
                    ProfileKey::create(key)
                });

                let profile_key = if let Some(k) = profile_key {
                    k
                } else {
                    // UploadProfile will generate a profile key if needed
                    client.send(UploadProfile).await.unwrap();
                    return;
                };

                if let Some(lpf) = &self_recipient.last_profile_fetch {
                    if Utc.from_utc_datetime(lpf) > Utc::now() - chrono::Duration::days(1) && !force
                    {
                        log::info!("Our own profile is up-to-date, not fetching.");
                        return;
                    }
                }

                let online = service
                    .retrieve_profile_by_id(ServiceAddress::from(uuid), Some(profile_key))
                    .await;

                let outdated = match online {
                    Ok(profile) => {
                        let unidentified_access_enabled = profile.unidentified_access.is_some();
                        let capabilities = profile.capabilities.clone();
                        client
                            .send(ProfileFetched(uuid, Some(profile)))
                            .await
                            .unwrap();

                        !unidentified_access_enabled
                            || capabilities != whisperfish_device_capabilities()
                    }
                    Err(e) => {
                        if let ServiceError::NotFoundError = e {
                            // No profile of ours online, let's upload one.
                            true
                        } else {
                            log::error!("Error fetching own profile: {}", e);
                            false
                        }
                    }
                };

                if outdated {
                    log::info!("Considering our profile as outdated, uploading new one.");
                    client.send(UploadProfile).await.unwrap();
                }
            }
            .into_actor(self)
            .map(|_, act, _| act.migration_state.notify_self_profile_ready()),
        )
    }
}

impl Handler<UpdateProfile> for ClientActor {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, new_profile: UpdateProfile, ctx: &mut Self::Context) -> Self::Result {
        let storage = self.storage.clone().unwrap();
        let client = ctx.address();
        let config = self.config.clone();
        let uuid = config.get_uuid().expect("valid uuid at this point");

        // XXX: Validate emoji character somehow
        Box::pin(async move {
            storage.update_profile_details(
                &uuid,
                &Some(new_profile.given_name),
                &Some(new_profile.family_name),
                &Some(new_profile.about),
                &Some(new_profile.emoji),
            );

            client.send(UploadProfile).await.unwrap();
            client.send(RefreshProfileAttributes).await.unwrap();
            client.send(MultideviceSyncProfile).await.unwrap();
            // XXX: Send a profile re-download message to contacts
        })
    }
}

impl Handler<UploadProfile> for ClientActor {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, _: UploadProfile, ctx: &mut Self::Context) -> Self::Result {
        let storage = self.storage.clone().unwrap();
        let service = self.authenticated_service();
        let client = ctx.address();
        let config = self.config.clone();
        let uuid = config.get_uuid().expect("valid uuid at this point");

        Box::pin(async move {
            let self_recipient = storage
                .fetch_self_recipient()
                .expect("self recipient should be set by now");
            let profile_key = self_recipient
                .profile_key
                .map(|bytes| {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&bytes);
                    ProfileKey::create(key)
                })
                .unwrap_or_else(|| {
                    log::info!("Generating profile key");
                    ProfileKey::generate(rand::thread_rng().gen())
                });
            let name = ProfileName {
                given_name: self_recipient.profile_given_name.as_deref().unwrap_or(""),
                family_name: self_recipient.profile_family_name.as_deref(),
            };

            let mut am = AccountManager::new(service, Some(profile_key));
            if let Err(e) = am
                .upload_versioned_profile_without_avatar(
                    uuid,
                    name,
                    self_recipient.about,
                    self_recipient.about_emoji,
                    true,
                )
                .await
            {
                log::error!("Error uploading profile: {}. Retrying in 60 seconds.", e);
                actix::spawn(async move {
                    actix::clock::sleep(std::time::Duration::from_secs(60)).await;
                    client
                        .send(UploadProfile)
                        .await
                        .expect("client still running");
                });

                return;
            }

            // Now also set the database
            storage.update_profile_key(
                None,
                Some(uuid),
                None,
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
    fn handle(&mut self, _: RefreshProfileAttributes, ctx: &mut Self::Context) -> Self::Result {
        log::info!("Sending profile attributes");

        let storage = self.storage.clone().unwrap();
        let service = self.authenticated_service();
        let address = ctx.address();

        Box::pin(async move {
            let registration_id = storage.get_local_registration_id(None).await.unwrap();
            let pni_registration_id = storage.get_local_pni_registration_id(None).await.unwrap();
            let self_recipient = storage.fetch_self_recipient().expect("self set by now");

            let profile_key = self_recipient.profile_key();
            let mut am = AccountManager::new(service, profile_key.map(ProfileKey::create));
            let unidentified_access_key = profile_key
                .map(ProfileKey::create)
                .as_ref()
                .map(ProfileKey::derive_access_key);

            let account_attributes = AccountAttributes {
                signaling_key: None,
                registration_id,
                voice: false,
                video: false,
                fetches_messages: true,
                pin: None,
                registration_lock: None,
                unidentified_access_key: unidentified_access_key.map(Vec::from),
                unrestricted_unidentified_access: false,
                discoverable_by_phone_number: true,
                capabilities: whisperfish_device_capabilities(),
                name: Some("Whisperfish".into()),
                pni_registration_id,
            };
            if let Err(e) = am.set_account_attributes(account_attributes).await {
                log::error!("Error refreshing profile attributes: {}", e);
                actix::spawn(async move {
                    actix::clock::sleep(std::time::Duration::from_secs(60)).await;
                    address
                        .send(UploadProfile)
                        .await
                        .expect("client still running");
                });
            } else {
                log::info!("Profile attributes refreshed");
            }
        })
    }
}
