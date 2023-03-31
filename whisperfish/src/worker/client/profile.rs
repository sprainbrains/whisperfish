use super::*;
use crate::worker::profile_refresh::OutdatedProfile;
use actix::prelude::*;
use libsignal_service::profile_cipher::ProfileCipher;
use libsignal_service::profile_service::ProfileService;
use libsignal_service::push_service::SignalServiceProfile;
use tokio::io::AsyncWriteExt;

impl StreamHandler<OutdatedProfile> for ClientActor {
    fn handle(&mut self, OutdatedProfile(uuid, key): OutdatedProfile, ctx: &mut Self::Context) {
        log::trace!("Received OutdatedProfile({}, [..]), fetching.", uuid);
        let mut service = if let Some(ws) = self.ws.clone() {
            ProfileService::from_socket(ws)
        } else {
            log::debug!("Ignoring outdated profiles until reconnected.");
            return;
        };

        // If our own Profile is outdated, schedule a profile refresh
        if self.config.get_uuid_clone() == uuid.to_string() {
            log::trace!("Scheduling a refresh for our own profile");
            ctx.notify(RefreshOwnProfile { force: false });
            return;
        }

        ctx.spawn(
            async move {
                (
                    uuid,
                    service
                        .retrieve_profile_by_id(ServiceAddress::from(uuid), key)
                        .await,
                )
            }
            .into_actor(self)
            .map(|(recipient_uuid, profile), _act, ctx| {
                match profile {
                    Ok(profile) => ctx.notify(ProfileFetched(recipient_uuid, Some(profile))),
                    Err(e) => {
                        if let ServiceError::NotFoundError = e {
                            ctx.notify(ProfileFetched(recipient_uuid, None))
                        } else {
                            log::error!("Error refreshing outdated profile: {}", e);
                        }
                    }
                };
            }),
        );
    }
}

/// Queue a force-refresh of a profile avatar
#[derive(Message)]
#[rtype(result = "()")]
pub struct RefreshProfileAvatar(uuid::Uuid);

impl Handler<RefreshProfileAvatar> for ClientActor {
    type Result = ();

    fn handle(
        &mut self,
        RefreshProfileAvatar(uuid): RefreshProfileAvatar,
        ctx: &mut Self::Context,
    ) {
        log::trace!("Received RefreshProfileAvatar(..), fetching.");
        let storage = self.storage.as_ref().unwrap();
        let recipient = {
            match storage.fetch_recipient_by_uuid(uuid) {
                Some(r) => r,
                None => {
                    log::error!("No recipient with uuid {}", uuid);
                    return;
                }
            }
        };
        let (avatar, key) = match recipient.signal_profile_avatar {
            Some(avatar) => (
                avatar,
                recipient.profile_key.expect("avatar comes with a key"),
            ),
            None => {
                log::error!(
                    "Recipient without avatar; not refreshing avatar: {:?}",
                    recipient
                );
                return;
            }
        };
        let mut service = self.authenticated_service();
        ctx.spawn(
            async move {
                let mut bytes = [0u8; 32];
                bytes.copy_from_slice(&key);
                let key = zkgroup::profiles::ProfileKey::create(bytes);
                let cipher = ProfileCipher::from(key);
                let mut avatar = service.retrieve_profile_avatar(&avatar).await?;
                // 10MB is what Signal Android allocates
                let mut contents = Vec::with_capacity(10 * 1024 * 1024);
                let len = avatar.read_to_end(&mut contents).await?;
                contents.truncate(len);
                Ok((uuid, cipher.decrypt_avatar(&contents)?))
            }
            .into_actor(self)
            .map(|res: anyhow::Result<_>, _act, ctx| {
                match res {
                    Ok((recipient_uuid, avatar)) => {
                        ctx.notify(ProfileAvatarFetched(recipient_uuid, avatar))
                    }
                    Err(e) => {
                        log::error!("Error fetching profile avatar: {}", e);
                    }
                };
            }),
        );
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct ProfileAvatarFetched(uuid::Uuid, Vec<u8>);

impl Handler<ProfileAvatarFetched> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(
        &mut self,
        ProfileAvatarFetched(uuid, bytes): ProfileAvatarFetched,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        Box::pin(
            async move {
                let settings = crate::config::SettingsBridge::default();
                let avatar_dir = settings.get_string("avatar_dir");
                let avatar_dir = Path::new(&avatar_dir);

                if !avatar_dir.exists() {
                    std::fs::create_dir(avatar_dir)?;
                }

                let out_path = avatar_dir.join(uuid.to_string());

                let mut f = tokio::fs::File::create(out_path).await?;
                f.write_all(&bytes).await?;

                Ok(())
            }
            .into_actor(self)
            .map(move |res: anyhow::Result<_>, _act, _ctx| {
                match res {
                    Ok(()) => {
                        // XXX this is basically incomplete.
                        // Storage should send out some NotifyRecipientUpdated
                    }
                    Err(e) => {
                        log::warn!("Error with fetched avatar: {}", e);
                    }
                }
            }),
        )
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub(super) struct ProfileFetched(pub uuid::Uuid, pub Option<SignalServiceProfile>);

impl Handler<ProfileFetched> for ClientActor {
    type Result = ();

    fn handle(
        &mut self,
        ProfileFetched(uuid, profile): ProfileFetched,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        match self.handle_profile_fetched(ctx, uuid, profile) {
            Ok(()) => {}
            Err(e) => {
                log::warn!("Error with fetched profile: {}", e);
            }
        }
    }
}

impl ClientActor {
    fn handle_profile_fetched(
        &mut self,
        ctx: &mut <Self as Actor>::Context,
        recipient_uuid: Uuid,
        profile: Option<SignalServiceProfile>,
    ) -> anyhow::Result<()> {
        log::info!("Fetched profile: {:?}", profile);
        let storage = self.storage.clone().unwrap();
        let recipient = storage
            .fetch_recipient_by_uuid(recipient_uuid)
            .ok_or_else(|| {
                anyhow::anyhow!("could not find recipient for which we fetched a profile")
            })?;
        let key = &recipient.profile_key;

        let mut db = storage.db();

        use crate::schema::recipients::dsl::*;
        use diesel::prelude::*;

        if let Some(profile) = profile {
            let cipher = if let Some(key) = key {
                let mut bytes = [0u8; 32];
                bytes.copy_from_slice(key);
                ProfileCipher::from(zkgroup::profiles::ProfileKey::create(bytes))
            } else {
                anyhow::bail!(
                    "Fetched a profile for a contact that did not share the profile key."
                );
            };

            let profile_decrypted = profile.decrypt(cipher)?;

            log::info!(
                "Decrypted profile {:?}.  Updating database.",
                profile_decrypted
            );

            if let Some(avatar) = &profile.avatar {
                if !avatar.is_empty() {
                    ctx.notify(RefreshProfileAvatar(recipient_uuid));
                }
            }

            let new_unidentified_identified_mode = if profile.unrestricted_unidentified_access {
                UnidentifiedAccessMode::Unrestricted
            } else {
                recipient.unidentified_access_mode
            };

            diesel::update(recipients)
                .set((
                    profile_given_name.eq(profile_decrypted.name.as_ref().map(|x| &x.given_name)),
                    profile_family_name.eq(profile_decrypted
                        .name
                        .as_ref()
                        .and_then(|x| x.family_name.as_ref())),
                    profile_joined_name.eq(profile_decrypted.name.as_ref().map(|x| x.to_string())),
                    about.eq(profile_decrypted.about),
                    about_emoji.eq(profile_decrypted.about_emoji),
                    unidentified_access_mode.eq(new_unidentified_identified_mode),
                    signal_profile_avatar.eq(profile.avatar),
                    last_profile_fetch.eq(Utc::now().naive_utc()),
                ))
                .filter(uuid.nullable().eq(&recipient_uuid.to_string()))
                .execute(&mut *db)
                .expect("db");
        } else {
            // XXX: We came here through 404 error, can that mean unregistered user?
            diesel::update(recipients)
                .set((last_profile_fetch.eq(Utc::now().naive_utc()),))
                .filter(uuid.nullable().eq(&recipient_uuid.to_string()))
                .execute(&mut *db)
                .expect("db");
        }
        // TODO For completeness, we should tickle the GUI for an update.

        Ok(())
    }
}
