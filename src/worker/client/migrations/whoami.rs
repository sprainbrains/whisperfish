use actix::prelude::*;

use super::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct WhoAmI;

impl Handler<WhoAmI> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, _: WhoAmI, _ctx: &mut Self::Context) -> Self::Result {
        let storage = self.storage.clone().unwrap();
        let cfg = storage.read_config().expect("read config");

        let mut service = self.authenticated_service();

        Box::pin(
            async move {
                if let Some(uuid) = cfg.uuid {
                    if !uuid.trim().is_empty() {
                        log::trace!("UUID is already set: {}", uuid);
                        return Ok(None);
                    }
                }

                let response = service.whoami().await?;

                Ok::<_, failure::Error>(Some(response.uuid))
            }
            .into_actor(self)
            .map(move |result: Result<Option<String>, _>, act, _ctx| {
                let uuid = match result {
                    Ok(Some(uuid)) => uuid,
                    Ok(None) => return,
                    Err(e) => {
                        log::error!("fetching UUID: {}", e);
                        return;
                    }
                };

                if let Some(credentials) = act.credentials.as_mut() {
                    credentials.uuid = Some(uuid.clone());
                    let mut cfg = storage.read_config().expect("read config");
                    cfg.uuid = Some(uuid);
                    storage.write_config(cfg).expect("write config");
                } else {
                    log::error!("Credentials was none while setting UUID");
                }
            }),
        )
    }
}
