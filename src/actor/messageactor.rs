use crate::gui::StorageReady;
use crate::model::message::MessageModel;
use crate::qmlapp::QmlApp;
use crate::store::{orm, Storage};
use crate::worker::ClientActor;

use actix::prelude::*;
use qmetaobject::prelude::*;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct FetchSession {
    pub id: i32,
    pub mark_read: bool,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct FetchMessage(pub i32);

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct FetchAllMessages(pub i32);

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct DeleteMessage(pub i32, pub usize);

#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
pub struct QueueMessage {
    pub e164: String,
    pub message: String,
    pub attachment: String,
}

#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
pub enum QueueGroupMessage {
    GroupV1Message {
        group_id: String,
        message: String,
        attachment: String,
    },
    GroupV2Message {
        group_id: String,
        message: String,
        attachment: String,
    },
}

#[derive(Message)]
#[rtype(result = "()")]
/// Send a ne
pub struct EndSession(pub String);

pub struct MessageActor {
    inner: QObjectBox<MessageModel>,
    storage: Option<Storage>,
    config: std::sync::Arc<crate::config::SignalConfig>,
}

pub fn pad_fingerprint(fp: &mut String) {
    if fp.len() == 60 {
        // twelve groups, eleven spaces.
        for i in 1..12 {
            fp.insert(6 * i - 1, ' ');
        }
    }
}

impl MessageActor {
    pub fn new(
        app: &mut QmlApp,
        client: Addr<ClientActor>,
        config: std::sync::Arc<crate::config::SignalConfig>,
    ) -> Self {
        let inner = QObjectBox::new(MessageModel::default());
        app.set_object_property("MessageModel".into(), inner.pinned());
        inner.pinned().borrow_mut().client_actor = Some(client);

        Self {
            inner,
            storage: None,
            config,
        }
    }
}

impl Actor for MessageActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.inner.pinned().borrow_mut().actor = Some(ctx.address());
    }
}

impl Handler<StorageReady> for MessageActor {
    type Result = ();

    fn handle(&mut self, storageready: StorageReady, _ctx: &mut Self::Context) -> Self::Result {
        self.storage = Some(storageready.storage);
        log::trace!("MessageActor has a registered storage");
    }
}

impl Handler<FetchSession> for MessageActor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(
        &mut self,
        FetchSession { id: sid, mark_read }: FetchSession,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let storage = self.storage.as_ref().unwrap().clone();
        let sess = storage
            .fetch_session_by_id(sid)
            .expect("FIXME No session returned!");
        let group_members = if sess.is_group_v1() {
            let group = sess.unwrap_group_v1();
            storage
                .fetch_group_members_by_group_v1_id(&group.id)
                .into_iter()
                .map(|(_, r)| r)
                .collect()
        } else if sess.is_group_v2() {
            let group = sess.unwrap_group_v2();
            storage
                .fetch_group_members_by_group_v2_id(&group.id)
                .into_iter()
                .map(|(_, r)| r)
                .collect()
        } else {
            Vec::new()
        };
        if mark_read {
            storage.mark_session_read(sess.id);
        }

        let myself = storage
            .fetch_self_recipient(&self.config)
            .expect("self recipient in db")
            .to_service_address();
        let sess_type = sess.r#type.clone();
        Box::pin(
            async move {
                if let orm::SessionType::DirectMessage(recipient) = sess_type {
                    use libsignal_service::prelude::protocol::SessionStoreExt;
                    log::info!("requested peer identity for {:?}; #303", recipient);
                    let addr = recipient.to_service_address();
                    match storage.compute_safety_number(&myself, &addr, None).await {
                        Ok(mut fp) => {
                            log::info!("Computed fingerprint {}", fp);
                            pad_fingerprint(&mut fp);
                            Some(fp)
                        }
                        Err(e) => {
                            log::warn!(
                                "FetchSession: returning None for peer_ident because {:?}",
                                e
                            );
                            None
                        }
                    }
                } else {
                    None
                }
            }
            .into_actor(self)
            .map(move |fingerprint, act, _ctx| {
                act.inner.pinned().borrow_mut().handle_fetch_session(
                    sess,
                    group_members,
                    fingerprint,
                );
            }),
        )
    }
}

impl Handler<FetchMessage> for MessageActor {
    type Result = ();

    fn handle(&mut self, FetchMessage(id): FetchMessage, _ctx: &mut Self::Context) -> Self::Result {
        let storage = self.storage.as_ref().unwrap();
        let message = storage
            .fetch_message_by_id(id)
            .unwrap_or_else(|| panic!("No message with id {}", id));
        let receipts = storage.fetch_message_receipts(message.id);
        let attachments = storage.fetch_attachments_for_message(message.id);
        let reactions = storage.fetch_reactions_for_message(message.id);
        let recipient = if let Some(id) = message.sender_recipient_id {
            self.storage.as_ref().unwrap().fetch_recipient_by_id(id)
        } else {
            None
        };
        self.inner.pinned().borrow_mut().handle_fetch_message(
            message,
            recipient,
            attachments,
            receipts,
            reactions,
        );
    }
}

impl Handler<FetchAllMessages> for MessageActor {
    type Result = ();

    fn handle(
        &mut self,
        FetchAllMessages(sid): FetchAllMessages,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let storage = self.storage.as_ref().unwrap();
        let messages = storage.fetch_all_messages_augmented(sid);

        self.inner
            .pinned()
            .borrow_mut()
            .handle_fetch_all_messages(messages);
    }
}

impl Handler<DeleteMessage> for MessageActor {
    type Result = ();

    fn handle(
        &mut self,
        DeleteMessage(id, idx): DeleteMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let del_rows = self.storage.as_ref().unwrap().delete_message(id);
        self.inner.pinned().borrow_mut().handle_delete_message(
            id,
            idx,
            del_rows.expect("FIXME no rows deleted"),
        );
    }
}

impl Handler<QueueGroupMessage> for MessageActor {
    type Result = ();

    fn handle(&mut self, msg: QueueGroupMessage, _ctx: &mut Self::Context) -> Self::Result {
        log::trace!("MessageActor::handle({:?})", msg);

        let storage = self.storage.as_mut().unwrap();

        let (group, message, attachment) = match msg {
            QueueGroupMessage::GroupV1Message {
                group_id,
                message,
                attachment,
            } => (
                storage
                    .fetch_session_by_group_v1_id(&group_id)
                    .expect("existing session"),
                message,
                attachment,
            ),
            QueueGroupMessage::GroupV2Message {
                group_id,
                message,
                attachment,
            } => (
                storage
                    .fetch_session_by_group_v2_id(&group_id)
                    .expect("existing session"),
                message,
                attachment,
            ),
        };

        let has_attachment = !attachment.is_empty();

        let (msg, _session) = storage.process_message(
            crate::store::NewMessage {
                session_id: None,
                source_e164: None,
                source_uuid: None,
                text: message,
                timestamp: chrono::Utc::now().naive_utc(),
                has_attachment,
                mime_type: if has_attachment {
                    Some(
                        mime_guess::from_path(&attachment)
                            .first_or_octet_stream()
                            .essence_str()
                            .into(),
                    )
                } else {
                    None
                },
                attachment: if has_attachment {
                    Some(attachment)
                } else {
                    None
                },
                flags: 0,
                outgoing: true,
                received: false,
                sent: false,
                is_read: true,
            },
            Some(group),
        );

        let attachments = storage.fetch_attachments_for_message(msg.id);

        self.inner
            .pinned()
            .borrow_mut()
            .handle_queue_message(msg, attachments);
    }
}

impl Handler<QueueMessage> for MessageActor {
    type Result = ();

    fn handle(&mut self, msg: QueueMessage, _ctx: &mut Self::Context) -> Self::Result {
        log::trace!("MessageActor::handle({:?})", msg);
        let storage = self.storage.as_mut().unwrap();

        let has_attachment = !msg.attachment.is_empty();

        let (msg, _session) = storage.process_message(
            crate::store::NewMessage {
                session_id: None,
                source_e164: Some(msg.e164),
                source_uuid: None,
                text: msg.message,
                timestamp: chrono::Utc::now().naive_utc(),
                has_attachment,
                mime_type: if has_attachment {
                    Some(
                        mime_guess::from_path(&msg.attachment)
                            .first_or_octet_stream()
                            .essence_str()
                            .into(),
                    )
                } else {
                    None
                },
                attachment: if has_attachment {
                    Some(msg.attachment)
                } else {
                    None
                },
                flags: 0,
                outgoing: true,
                received: false,
                sent: false,
                is_read: true,
            },
            None,
        );

        let attachments = storage.fetch_attachments_for_message(msg.id);

        self.inner
            .pinned()
            .borrow_mut()
            .handle_queue_message(msg, attachments);
    }
}

impl Handler<EndSession> for MessageActor {
    type Result = ();

    fn handle(&mut self, EndSession(e164): EndSession, _ctx: &mut Self::Context) -> Self::Result {
        use libsignal_service::content::DataMessageFlags;
        log::trace!("MessageActor::EndSession({})", e164);

        let storage = self.storage.as_mut().unwrap();

        let (msg, _session) = storage.process_message(
            crate::store::NewMessage {
                session_id: None,
                source_e164: Some(e164),
                source_uuid: None,
                text: "[Whisperfish] Reset secure session".into(),
                timestamp: chrono::Utc::now().naive_utc(),
                has_attachment: false,
                mime_type: None,
                attachment: None,
                flags: DataMessageFlags::EndSession.into(),
                outgoing: true,
                received: false,
                sent: false,
                is_read: true,
            },
            None,
        );

        self.inner
            .pinned()
            .borrow_mut()
            .handle_queue_message(msg, vec![]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pad_fingerprint_smoke() {
        let mut s = "892064087450853131489552767731995657884565179277972848560834".to_string();
        pad_fingerprint(&mut s);
        assert_eq!(
            s,
            "89206 40874 50853 13148 95527 67731 99565 78845 65179 27797 28485 60834"
        );
    }
}
