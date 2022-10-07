use actix::prelude::*;
use libsignal_service::{profile_cipher::ProfileCipher, push_service::SignalServiceProfile};

use crate::worker::profile_refresh::OutdatedProfile;

use super::*;

impl StreamHandler<OutdatedProfile> for ClientActor {
    fn handle(&mut self, OutdatedProfile(uuid, key): OutdatedProfile, ctx: &mut Self::Context) {
        let mut service = self.authenticated_service();
        ctx.spawn(
            async move {
                (
                    uuid,
                    service
                        .retrieve_profile_by_id(ServiceAddress::from(uuid), Some(key))
                        .await,
                )
            }
            .into_actor(self)
            .map(|(recipient_uuid, profile), _act, ctx| {
                match profile {
                    Ok(profile) => ctx.notify(ProfileFetched(recipient_uuid, profile)),
                    Err(e) => {
                        log::error!("During profile fetch: {}", e);
                    }
                };
            }),
        );
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
struct ProfileFetched(uuid::Uuid, SignalServiceProfile);

impl Handler<ProfileFetched> for ClientActor {
    type Result = ();

    fn handle(
        &mut self,
        ProfileFetched(uuid, profile): ProfileFetched,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        match self.handle_profile_fetched(uuid, profile) {
            Ok(()) => (),
            Err(e) => {
                log::warn!("Error with fetched profile: {}", e);
            }
        }
    }
}

impl ClientActor {
    fn handle_profile_fetched(
        &mut self,
        recipient_uuid: Uuid,
        profile: SignalServiceProfile,
    ) -> anyhow::Result<()> {
        log::info!("Fetched profile: {:?}", profile);
        let storage = self.storage.clone().unwrap();
        let db = storage.db.lock();

        use crate::schema::recipients::dsl::*;
        use diesel::prelude::*;

        let key: Option<Vec<u8>> = recipients
            .select(profile_key)
            .filter(uuid.nullable().eq(&recipient_uuid.to_string()))
            .first(&*db)
            .expect("db");
        let cipher = if let Some(key) = key {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&key);
            ProfileCipher::from(zkgroup::profiles::ProfileKey::create(bytes))
        } else {
            anyhow::bail!("Fetched a profile for a contact that did not share the profile key.");
        };

        let name = profile.name.map(|x| cipher.decrypt_name(x)).transpose()?;
        let about = profile.about.map(|x| cipher.decrypt_about(x)).transpose()?;
        let emoji = profile
            .about_emoji
            .map(|x| cipher.decrypt_emoji(x))
            .transpose()?;

        log::info!("Decrypted profile {:?} {:?} {:?}", name, about, emoji);

        Ok(())
    }
}
