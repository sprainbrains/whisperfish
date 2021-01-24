use super::*;

impl ClientWorker {
    // method called from Qt
    pub fn reload_linked_devices(&self) {
        let actor = self.actor.clone().unwrap();
        Arbiter::spawn(async move {
            if let Err(e) = actor.send(ReloadLinkedDevices).await {
                log::error!("{:?}", e);
            }
        })
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ReloadLinkedDevices;

impl Handler<ReloadLinkedDevices> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _: ReloadLinkedDevices, _ctx: &mut Self::Context) -> Self::Result {
        log::trace!("handle(ReloadLinkedDevices)");

        let mut service = self.authenticated_service();

        Box::pin(
            // Without `async move`, service would be borrowed instead of encapsulated in a Future.
            async move { service.devices().await }.into_actor(self).map(
                move |result, act, _ctx| {
                    match result {
                        Err(e) => {
                            // XXX show error
                            log::error!("Refresh linked devices failed: {}", e);
                        }
                        Ok(devices) => {
                            log::trace!("Successfully refreshed linked devices: {:?}", devices);
                            // A bunch bindings because of scope
                            let client_worker = act.inner.pinned();
                            let client_worker = client_worker.borrow_mut();
                            let device_model =
                                client_worker.device_model.as_ref().unwrap().pinned();
                            device_model.borrow_mut().set_devices(devices);
                        }
                    }
                },
            ),
        )
    }
}
