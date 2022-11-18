use super::*;
use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct WhoAmI;

impl Handler<WhoAmI> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, _: WhoAmI, _ctx: &mut Self::Context) -> Self::Result {
        let mut service = self.authenticated_service();
        let config = std::sync::Arc::clone(&self.config);
        let config2 = std::sync::Arc::clone(&self.config);

        Box::pin(
            async move {
                if !config.get_uuid_clone().is_empty() {
                    log::trace!("UUID is already set: {}", config.get_uuid_clone());
                    return Ok(None);
                }

                let response = service.whoami().await?;

                Ok::<_, anyhow::Error>(Some(response.uuid))
            }
            .into_actor(self)
            .map(move |result: Result<Option<uuid::Uuid>, _>, act, _ctx| {
                let uuid = match result {
                    Ok(Some(uuid)) => uuid,
                    Ok(None) => return,
                    Err(e) => {
                        log::error!("fetching UUID: {}", e);
                        return;
                    }
                };

                if let Some(credentials) = act.credentials.as_mut() {
                    credentials.uuid = Some(uuid);
                    config2.set_uuid(uuid.to_string());
                    config2.write_to_file().expect("write config");
                } else {
                    log::error!("Credentials was none while setting UUID");
                }

                act.migration_state.notify_whoami();
            }),
        )
    }
}
