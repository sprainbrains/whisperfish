use std::collections::HashMap;

use crate::model::*;
use crate::sfos::*;
use crate::store::{Storage, StorageReady};
use crate::model::session;

use actix::prelude::*;
use diesel::prelude::*;
use qmetaobject::*;

#[derive(QObject, Default)]
#[allow(non_snake_case)]  // XXX: QML expects these as-is; consider changing later
struct MessageModel {
    base: qt_base_class!(trait QAbstractListModel),
    actor: Option<Addr<MessageActor>>,

    messages: Vec<Message>,

    peerIdentity: qt_property!(QString; NOTIFY peerIdentityChanged),
    peerName: qt_property!(QString; NOTIFY peerNameChanged),
    peerTel: qt_property!(QString; NOTIFY peerTelChanged),
    groupMembers: qt_property!(QString; NOTIFY groupMembersChanged),
    sessionId: qt_property!(i64; NOTIFY sessionIdChanged),
    group: qt_property!(bool; NOTIFY groupChanged),

    peerIdentityChanged: qt_signal!(),
    peerNameChanged: qt_signal!(),
    peerTelChanged: qt_signal!(),
    groupMembersChanged: qt_signal!(),
    sessionIdChanged: qt_signal!(),
    groupChanged: qt_signal!(),

    load: qt_method!(fn(&self, sid: i64, peer_name: QString)),
}

struct FetchSession(i64);
impl actix::Message for FetchSession {
    type Result = ();
}

struct FetchAllMessages(i64);
impl actix::Message for FetchAllMessages {
    type Result = ();
}

pub struct MessageActor {
    inner: QObjectBox<MessageModel>,
    storage: Option<Storage>,
}

#[derive(Queryable)]
pub struct Message {
    pub id: i32,
    pub sid: i64,
    pub source: String,
    pub message: String,  // NOTE: "text" in schema, doesn't apparently matter
    pub timestamp: i64,
    pub sent: bool,
    pub received: bool,
    pub flags: i32,
    pub attachment: Option<String>,
    pub mimetype: Option<String>,
    pub hasattachment: bool,
    pub outgoing: bool,
    // pub queued: bool, // TODO Used only by LEFT OUTER JOIN - implement that
}

impl MessageActor {
    pub fn new(app: &mut SailfishApp) -> Self {
        let inner = QObjectBox::new(MessageModel::default());
        app.set_object_property("MessageModel".into(), inner.pinned());

        Self {
            inner,
            storage: None,
        }
    }
}

impl Actor for MessageActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.inner.pinned().borrow_mut().actor = Some(ctx.address());
    }
}

define_model_roles! {
    enum MessageRoles for Message {
        SID(sid):                                 "sid",
        Source(source via QString::from):         "source",
        Message(message via QString::from):       "message",
        Timestamp(timestamp):                     "timestamp",
        Outgoing(outgoing):                       "outgoing",
        Sent(sent):                               "sent",
        Received(received):                       "received",
        Attachment(attachment via qstring_from_option): "attachment",
        MimeType(mimetype via qstring_from_option):     "mimetype",
        HasAttachment(hasattachment):             "hasattachment",
        Flags(flags):                             "flags",
        // Queued(queued):                           "queued",
    }
}

impl MessageModel {
    fn load(&mut self, sid: i64, peer_name: QString) {
        (self as &mut dyn QAbstractListModel).begin_reset_model();

        self.messages.clear();

        (self as &mut dyn QAbstractListModel).end_reset_model();

        use futures::prelude::*;
        Arbiter::spawn(self.actor.as_ref().unwrap().send(FetchSession(sid)).map(Result::unwrap));
        log::trace!("Dispatched FetchSession({})", sid);
    }

    fn handle_fetch_session(&mut self, sess: session::Session) {
        log::trace!("handle_fetch_session({})", sess.message);
        self.sessionId = sess.id;
        self.sessionIdChanged();

        self.group = sess.is_group;
        self.groupChanged();

        let group_name = sess.group_name.unwrap_or(String::new());
        if sess.is_group && group_name != "" {
            self.peerName = QString::from(group_name);
        } else {
            self.peerName = QString::from(sess.source.clone());
        }
        self.peerNameChanged();

        self.peerTel = QString::from(sess.source);
        self.peerTelChanged();

        self.groupMembers = QString::from(sess.group_members.unwrap_or(String::new()));
        self.groupMembersChanged();

        // TODO: contact identity key
        use futures::prelude::*;
        Arbiter::spawn(self.actor.as_ref().unwrap().send(FetchAllMessages(sess.id)).map(Result::unwrap));
        log::trace!("Dispatched FetchAllMessages({})", sess.id);
    }

    pub fn handle_fetch_all_messages(&mut self, messages: Vec<Message>) {
        log::trace!("handle_fetch_all_messages({}) count {}", messages[0].sid, messages.len());

        (self as &mut dyn QAbstractListModel).begin_insert_rows(0, messages.len() as i32);

        self.messages.extend(messages);

        (self as &mut dyn QAbstractListModel).end_insert_rows();
    }
}

impl QAbstractListModel for MessageModel {
    fn row_count(&self) -> i32 {
        self.messages.len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let role = MessageRoles::from(role);
        role.get(&self.messages[index.row() as usize])
    }

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        MessageRoles::role_names()
    }
}

impl Handler<StorageReady> for MessageActor {
    type Result = ();

    fn handle(
        &mut self,
        StorageReady(storage): StorageReady,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        self.storage = Some(storage);
        log::trace!("MessageActor has a registered storage");
    }
}

impl Handler<FetchSession> for MessageActor {
    type Result = ();

    fn handle(&mut self,
              FetchSession(sid): FetchSession,
              _ctx: &mut Self::Context
    ) -> Self::Result {
        let sess = self.storage.as_ref().unwrap().fetch_session(sid);
        self.inner.pinned().borrow_mut().handle_fetch_session(sess.expect("FIXME No session returned!"));
    }
}

impl Handler<FetchAllMessages> for MessageActor {
    type Result = ();

    fn handle(&mut self,
              FetchAllMessages(sid): FetchAllMessages,
              _ctx: &mut Self::Context
    ) -> Self::Result {
        let messages = self.storage.as_ref().unwrap().fetch_all_messages(sid);
        self.inner.pinned().borrow_mut().handle_fetch_all_messages(messages.expect("death"));
    }
}
