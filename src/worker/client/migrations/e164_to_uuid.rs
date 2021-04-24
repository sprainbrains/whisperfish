use actix::prelude::*;
use libsignal_protocol::stores::{IdentityKeyStore, SessionStore};
use libsignal_protocol::Address;

use super::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct E164ToUuid;

impl Handler<E164ToUuid> for ClientActor {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, _: E164ToUuid, _ctx: &mut Self::Context) -> Self::Result {
        let storage = self.storage.clone().unwrap();
        let config = std::sync::Arc::clone(&self.config);

        // Stuff to migrate:
        // 1. The session with yourself.
        // 2. The sessions with all e164-known recipients.
        // 2. The identities with all e164-known recipients.

        Box::pin(async move {
            if config.get_uuid_clone().is_empty() {
                log::error!("We don't have our own UUID yet. Let's retry at the next start.");
                return;
            }

            let recipients = storage.fetch_recipients();
            for recipient in recipients {
                if let (Some(e164), Some(uuid)) = (recipient.e164, recipient.uuid) {
                    // Look for sessions based on this e164
                    for sub_device_session in storage
                        .get_sub_device_sessions(e164.as_bytes())
                        .expect("storage")
                        .into_iter()
                        .chain(std::iter::once(1))
                    {
                        let e164_addr = Address::new(&e164, sub_device_session);
                        let uuid_addr = Address::new(&uuid, sub_device_session);

                        if let Some(e164_session) =
                            storage.load_session(e164_addr.clone()).expect("storage")
                        {
                            log::info!(
                                "Found an old E164-style session for {}. Migrating to {}",
                                e164,
                                uuid
                            );
                            if storage
                                .contains_session(uuid_addr.clone())
                                .expect("storage")
                            {
                                log::error!("Already found a session for {}_{}. Refusing to overwrite. Please file a bug report.", uuid, sub_device_session);
                            } else {
                                storage
                                    .store_session(uuid_addr.clone(), e164_session)
                                    .expect("storage");
                                SessionStore::delete_session(&storage, e164_addr.clone())
                                    .expect("storage");
                            }
                        }

                        if let Some(e164_identity) =
                            storage.get_identity(e164_addr.clone()).expect("storage")
                        {
                            log::info!(
                                "Found an old E164-style identity for {}. Migrating to {}",
                                e164,
                                uuid
                            );
                            if storage
                                .get_identity(uuid_addr.clone())
                                .expect("storage")
                                .is_some()
                            {
                                log::error!("Already found an identity for {}. Refusing to overwrite. Please upvote issue #326.", uuid);
                            } else {
                                storage
                                    .save_identity(uuid_addr, e164_identity.as_slice())
                                    .expect("storage");
                                storage.delete_identity(e164_addr).expect("storage");
                            }
                        }
                    }
                }
            }
        })
    }
}
