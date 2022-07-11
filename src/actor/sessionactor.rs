use std::collections::VecDeque;

use crate::actor::{FetchSession, UpdateSession};

use crate::gui::StorageReady;
use crate::model::session::SessionModel;
use crate::qmlapp::QmlApp;
use crate::store::{orm, Storage};

use actix::prelude::*;
use libsignal_protocol::ProtocolAddress;
use qmetaobject::prelude::*;

mod typing_notifications;
pub use typing_notifications::*;

#[derive(Message)]
#[rtype(result = "()")]
#[allow(clippy::type_complexity)]
struct SessionsLoaded(
    Vec<(
        orm::Session,
        Vec<orm::Recipient>,
        Option<(
            orm::Message,
            Vec<orm::Attachment>,
            Vec<(orm::Receipt, orm::Recipient)>,
        )>,
    )>,
);

#[derive(actix::Message)]
#[rtype(result = "()")]
// XXX this should be called *per message* instead of per session,
//     probably.
pub struct MarkSessionRead {
    pub sid: i32,
    pub already_unread: bool,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct MarkSessionMuted {
    pub sid: i32,
    pub muted: bool,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct MarkSessionArchived {
    pub sid: i32,
    pub archived: bool,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct MarkSessionPinned {
    pub sid: i32,
    pub pinned: bool,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct DeleteSession {
    pub id: i32,
    pub idx: usize,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct LoadAllSessions;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct RemoveIdentities {
    pub session_id: i32,
}

pub struct SessionActor {
    inner: QObjectBox<SessionModel>,
    storage: Option<Storage>,

    typing_queue: VecDeque<TypingQueueItem>,
}

impl SessionActor {
    pub fn new(app: &mut QmlApp) -> Self {
        let inner = QObjectBox::new(SessionModel::default());
        app.set_object_property("SessionModel".into(), inner.pinned());

        Self {
            inner,
            storage: None,
            typing_queue: VecDeque::new(),
        }
    }
}

impl Actor for SessionActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.inner.pinned().borrow_mut().actor = Some(ctx.address());
    }
}

impl Handler<StorageReady> for SessionActor {
    type Result = ();

    fn handle(&mut self, storageready: StorageReady, ctx: &mut Self::Context) -> Self::Result {
        self.storage = Some(storageready.storage);
        log::trace!("SessionActor has a registered storage");

        ctx.notify(LoadAllSessions);
    }
}

impl Handler<SessionsLoaded> for SessionActor {
    type Result = ();

    fn handle(
        &mut self,
        SessionsLoaded(sessions): SessionsLoaded,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let inner = self.inner.pinned();
        let mut inner = inner.borrow_mut();

        inner.handle_sessions_loaded(sessions);
    }
}

impl Handler<FetchSession> for SessionActor {
    type Result = ();

    fn handle(
        &mut self,
        FetchSession { id: sid, mark_read }: FetchSession,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let storage = self.storage.as_ref().unwrap();
        let sess = storage.fetch_session_by_id(sid).expect("existing session");
        let message = storage
            .fetch_last_message_by_session_id(sid)
            .expect("> 0 messages per session");
        let receipts = storage.fetch_message_receipts(message.id);
        let attachments = storage.fetch_attachments_for_message(message.id);

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

        self.inner.pinned().borrow_mut().handle_fetch_session(
            sess,
            group_members,
            message,
            attachments,
            receipts,
            mark_read,
        );
    }
}

impl Handler<UpdateSession> for SessionActor {
    type Result = ();

    fn handle(
        &mut self,
        UpdateSession { id: sid }: UpdateSession,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let storage = self.storage.as_ref().unwrap();
        let sess = storage.fetch_session_by_id(sid).expect("existing session");
        let message = storage
            .fetch_last_message_by_session_id(sid)
            .expect("> 0 messages per session");
        let receipts = storage.fetch_message_receipts(message.id);
        let attachments = storage.fetch_attachments_for_message(message.id);

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

        self.inner.pinned().borrow_mut().handle_update_session(
            sess,
            group_members,
            message,
            attachments,
            receipts,
        );
    }
}

impl Handler<MarkSessionRead> for SessionActor {
    type Result = ();

    fn handle(
        &mut self,
        MarkSessionRead {
            sid,
            already_unread,
        }: MarkSessionRead,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.storage.as_ref().unwrap().mark_session_read(sid);
        self.inner
            .pinned()
            .borrow_mut()
            .handle_mark_session_read(sid, already_unread);
    }
}

impl Handler<MarkSessionArchived> for SessionActor {
    type Result = ();

    fn handle(
        &mut self,
        MarkSessionArchived { sid, archived }: MarkSessionArchived,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.storage
            .as_ref()
            .unwrap()
            .mark_session_archived(sid, archived);
        self.inner
            .pinned()
            .borrow_mut()
            .handle_mark_session_archived(sid, archived);
    }
}

impl Handler<MarkSessionPinned> for SessionActor {
    type Result = ();

    fn handle(
        &mut self,
        MarkSessionPinned { sid, pinned }: MarkSessionPinned,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.storage
            .as_ref()
            .unwrap()
            .mark_session_pinned(sid, pinned);
        self.inner
            .pinned()
            .borrow_mut()
            .handle_mark_session_pinned(sid, pinned);
    }
}

impl Handler<MarkSessionMuted> for SessionActor {
    type Result = ();

    fn handle(
        &mut self,
        MarkSessionMuted { sid, muted }: MarkSessionMuted,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.storage
            .as_ref()
            .unwrap()
            .mark_session_muted(sid, muted);
        self.inner
            .pinned()
            .borrow_mut()
            .handle_mark_session_muted(sid, muted);
    }
}

impl Handler<DeleteSession> for SessionActor {
    type Result = ();

    fn handle(
        &mut self,
        DeleteSession { id, idx }: DeleteSession,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.storage.as_ref().unwrap().delete_session(id);

        self.inner.pinned().borrow_mut().handle_delete_session(idx);
    }
}

impl Handler<RemoveIdentities> for SessionActor {
    type Result = ();

    fn handle(
        &mut self,
        RemoveIdentities { session_id }: RemoveIdentities,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let storage = self.storage.as_ref().unwrap();
        let session = if let Some(s) = storage.fetch_session_by_id(session_id) {
            s
        } else {
            log::warn!(
                "Requested removal of identities for session {}, but session not found.",
                session_id
            );
            return;
        };
        let recipient = match session.r#type {
            orm::SessionType::DirectMessage(r) => r,
            orm::SessionType::GroupV1(_) => {
                log::warn!("Cannot remove identities for a group v1");
                return;
            }
            orm::SessionType::GroupV2(_) => {
                log::warn!("Cannot remove identities for a group v1");
                return;
            }
        };

        let identities = match (recipient.e164, recipient.uuid) {
            (None, None) => {
                log::debug!("No identities to remove");
                return;
            }
            (None, Some(uuid)) => vec![uuid],
            (Some(e164), None) => vec![e164],
            (Some(e164), Some(uuid)) => vec![e164, uuid],
        };
        for identity in identities {
            log::debug!("Removing identity {}", identity);
            let addr = ProtocolAddress::new(identity, 1);
            if !storage.delete_identity_key(&addr) {
                log::warn!("Removing identity key failed somehow.  Please file a bug.");
            }
        }
    }
}

impl Handler<LoadAllSessions> for SessionActor {
    type Result = ();

    /// Panics when storage is not yet set.
    fn handle(&mut self, _: LoadAllSessions, ctx: &mut Self::Context) {
        let session_actor = ctx.address();
        let storage = self.storage.clone().unwrap();

        actix::spawn(async move {
            let sessions = tokio::task::spawn_blocking(move || -> Result<_, anyhow::Error> {
                let sessions: Vec<orm::Session> = storage.fetch_sessions();
                let result = sessions
                    .into_iter()
                    .map(|session| {
                        let group_members = if session.is_group_v1() {
                            let group = session.unwrap_group_v1();
                            storage
                                .fetch_group_members_by_group_v1_id(&group.id)
                                .into_iter()
                                .map(|(_, r)| r)
                                .collect()
                        } else if session.is_group_v2() {
                            let group = session.unwrap_group_v2();
                            storage
                                .fetch_group_members_by_group_v2_id(&group.id)
                                .into_iter()
                                .map(|(_, r)| r)
                                .collect()
                        } else {
                            Vec::new()
                        };

                        let last_message = if let Some(last_message) =
                            storage.fetch_last_message_by_session_id(session.id)
                        {
                            last_message
                        } else {
                            return (session, group_members, None);
                        };
                        let last_message_receipts = storage.fetch_message_receipts(last_message.id);
                        let last_message_attachments =
                            storage.fetch_attachments_for_message(last_message.id);

                        (
                            session,
                            group_members,
                            Some((
                                last_message,
                                last_message_attachments,
                                last_message_receipts,
                            )),
                        )
                    })
                    .collect();
                Ok(result)
            })
            .await
            .expect("threadpool")
            .expect("fetch all sessions");
            // XXX handle error

            session_actor.send(SessionsLoaded(sessions)).await.unwrap();
            // XXX handle error
        });
    }
}
