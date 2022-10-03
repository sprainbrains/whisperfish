use actix::prelude::*;
use libsignal_service::push_service::SignalServiceProfile;

use crate::worker::profile_refresh::OutdatedProfile;

use super::*;

impl StreamHandler<OutdatedProfile> for ClientActor {
    fn handle(&mut self, OutdatedProfile(uuid): OutdatedProfile, ctx: &mut Self::Context) {
        let mut service = self.authenticated_service();
        ctx.spawn(
            async move { service.retrieve_profile_by_id(&uuid.to_string()).await }
                .into_actor(self)
                .map(|profile, _act, ctx| {
                    match profile {
                        Ok(profile) => ctx.notify(ProfileFetched(profile)),
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
struct ProfileFetched(SignalServiceProfile);

impl Handler<ProfileFetched> for ClientActor {
    type Result = ();

    fn handle(
        &mut self,
        ProfileFetched(profile): ProfileFetched,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        log::info!("Fetched profile: {:?}", profile);
    }
}
