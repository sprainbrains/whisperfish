use std::collections::HashMap;

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
    pub sid: i64,
    pub source: String,
    pub message: String,
    pub timestamp: u64,
    pub outgoing: bool,
    pub sent: bool,
    pub received: bool,
    pub attachment: String,
    pub mimetype: String,
    pub hasattachment: bool,
    pub flags: u32,
    pub queued: bool,
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
        Attachment(attachment via QString::from): "attachment",
        MimeType(mimetype via QString::from):     "mimetype",
        HasAttachment(hasattachment):             "hasattachment",
        Flags(flags):                             "flags",
        Queued(queued):                           "queued",
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

    pub fn handle_fetch_all_messages(&mut self, messages: ()) {
        // log::trace!("handle_fetch_all_messages({})", messages.unwrap().sid);
        log::trace!("handle_fetch_all_messages(SID)");
        // TODO: fetch all messages
        self.messages.push(Message {
            sid: 2,
            source: String::from("CONTACT_PHONE_NUMBER_HERE"),
            message: String::from("WHISPERFISH!"),
            timestamp: 1588934301257,
            outgoing: true,
            sent: true,
            received: true,
            hasattachment: false,
            attachment: String::new(),
            mimetype: String::new(),
            flags: 0,
            queued: false,
        });
        // TODO: begin insert, push messages, end insert
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
        self.inner.pinned().borrow_mut().handle_fetch_all_messages(messages);
    }
}
