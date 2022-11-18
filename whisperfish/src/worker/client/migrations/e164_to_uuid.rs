use super::*;
use actix::prelude::*;
use libsignal_service::prelude::protocol::{
    IdentityKeyStore, ProtocolAddress, SessionStore, SessionStoreExt,
};

#[derive(Message)]
#[rtype(result = "()")]
pub struct E164ToUuid;

impl Handler<E164ToUuid> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _: E164ToUuid, _ctx: &mut Self::Context) -> Self::Result {
        let mut storage = self.storage.clone().unwrap();
        let config = std::sync::Arc::clone(&self.config);

        let self_uuid_is_known = self.migration_state.self_uuid_is_known();
        let protocol_store_ready = self.migration_state.protocol_store_in_db();

        // Stuff to migrate:
        // 1. The session with yourself.
        // 2. The sessions with all e164-known recipients.
        // 2. The identities with all e164-known recipients.

        Box::pin(async move {
            // We need to wait until we know our own UUID and until the protocol store is ready
            self_uuid_is_known.await;
            protocol_store_ready.await;

            if config.get_uuid_clone().is_empty() {
                log::error!("We don't have our own UUID yet. Let's retry at the next start.");
                return;
            }

            let recipients = storage.fetch_recipients();
            for recipient in recipients {
                if let (Some(e164), Some(uuid)) = (recipient.e164, recipient.uuid) {
                    // Look for sessions based on this e164
                    for sub_device_session in storage
                        .get_sub_device_sessions(&e164)
                        .await
                        .expect("storage")
                        .into_iter()
                        .chain(std::iter::once(1))
                    {
                        let e164_addr =
                            ProtocolAddress::new(e164.clone(), sub_device_session.into());
                        let uuid_addr =
                            ProtocolAddress::new(uuid.clone(), sub_device_session.into());

                        if let Some(e164_session) = storage
                            .load_session(&e164_addr, None)
                            .await
                            .expect("storage")
                        {
                            log::info!(
                                "Found an old E164-style session for {}. Migrating to {}",
                                e164,
                                uuid
                            );
                            if storage
                                .load_session(&uuid_addr, None)
                                .await
                                .expect("storage")
                                .is_some()
                            {
                                // XXX At this point, we are not necessarily connected to the
                                // websocket.
                                // This means that we cannot programmatically trigger an EndSession
                                // from here.  Whenever we figure out to *correctly* queue
                                // messages
                                // (https://gitlab.com/whisperfish/whisperfish/-/issues/282), we
                                // can actually trigger a full session reset here.
                                //
                                // Our workaround consists of logging a warning, writing a "pseudo
                                // message" in the session, and keeping the issue open.
                                log::error!("Already found a session for {}_{}. This is a problem and may mean losing messages. Use the \"End session\" functionality in a direct message with {}, and upvote issue #336.", uuid, sub_device_session, e164);
                                storage.process_message(crate::store::NewMessage {
                                    attachment: None,
                                    flags: 0, // TODO: make this EndSession
                                    has_attachment: false,
                                    session_id: None,
                                    source_e164: Some(e164.clone()),
                                    source_uuid: Some(uuid.clone()),
                                    text: "[Whisperfish WARN] You somehow got issue #336 (https://gitlab.com/whisperfish/whisperfish/-/issues/336). Use the \"End session\" functionality in this session; you may otherwise fail to send or receive messages with this person.  This message will be repeated on every start of Whisperfish.".into(),
                                    timestamp: chrono::Utc::now().naive_utc(),
                                    sent: false,
                                    received: true,
                                    is_read: false,
                                    mime_type: None,
                                    outgoing: false,
                                    is_unidentified: false,
                                    quote_timestamp: None,
                                }, None);
                            } else {
                                storage
                                    .store_session(&uuid_addr, &e164_session, None)
                                    .await
                                    .expect("storage");
                                SessionStoreExt::delete_session(&storage, &e164_addr)
                                    .await
                                    .expect("storage");
                            }
                        }

                        if let Some(e164_identity) = storage
                            .get_identity(&e164_addr, None)
                            .await
                            .expect("storage")
                        {
                            log::info!(
                                "Found an old E164-style identity for {}. Migrating to {}",
                                e164,
                                uuid
                            );
                            if let Some(uuid_identity) = storage
                                .get_identity(&uuid_addr, None)
                                .await
                                .expect("storage")
                            {
                                if uuid_identity == e164_identity {
                                    log::trace!(
                                        "Found equal identities for {}/{}. Dropping E164.",
                                        e164,
                                        uuid
                                    );
                                } else {
                                    log::warn!("Found unequal identities for {}/{}. Refusing to overwrite; dropping E164.", e164, uuid);
                                }
                            } else {
                                log::trace!(
                                    "Found no UUID identity for {}. Moving to {}.",
                                    e164,
                                    uuid
                                );
                                storage
                                    .save_identity(&uuid_addr, &e164_identity, None)
                                    .await
                                    .expect("storage");
                            }
                            storage.delete_identity(&e164_addr).await.expect("storage");
                        }
                    }
                }
            }
        }
        .into_actor(self)
        .map(move |(), act, _ctx| {
            act.migration_state.notify_e164_to_uuid();
        }))
    }
}
