#![allow(non_snake_case)]

use std::collections::HashMap;

use crate::actor;
use crate::model::*;
use crate::store::Session;

use actix::prelude::*;
use qmetaobject::*;

#[derive(QObject, Default)]
pub struct SessionModel {
    base: qt_base_class!(trait QAbstractListModel),
    pub actor: Option<Addr<actor::SessionActor>>,

    content: Vec<Session>,

    count: qt_method!(fn(&self) -> i32),
    add: qt_method!(fn(&self, id: i64, mark_read: bool)),
    remove: qt_method!(fn(&self, idx: usize)),
    removeById: qt_method!(fn(&self, id: i64)),
    reload: qt_method!(fn(&self)),

    markRead: qt_method!(fn(&self, id: usize)),
    markReceived: qt_method!(fn(&self, id: usize)),
    markSent: qt_method!(fn(&self, id: usize, message: QString)),
}

impl SessionModel {
    fn count(&self) -> i32 {
        unimplemented!();
    }

    fn add(&self, id: i64, mark_read: bool) {
        unimplemented!();
    }

    fn remove(&self, idx: usize) {
        unimplemented!();
    }

    fn removeById(&self, id: i64) {
        unimplemented!();
    }

    fn reload(&self) {
        unimplemented!();
    }

    fn markRead(&self, _id: usize) {
        log::trace!("STUB: Mark read called");
        // XXX: don't forget sync messages
    }

    fn markReceived(&self, _id: usize) {
        log::trace!("STUB: Mark received called");
        // XXX: don't forget sync messages
    }

    fn markSent(&self, _id: usize, _message: QString) {
        log::trace!("STUB: Mark sent called");
        // XXX: don't forget sync messages
    }

    // Event handlers below this line

    /// Handle loaded session
    pub fn handle_sessions_loaded(&mut self, sessions: Vec<Session>) {
        // XXX: maybe this should be called before even accessing the db?
        (self as &mut dyn QAbstractListModel).begin_reset_model();
        self.content = sessions;
        (self as &mut dyn QAbstractListModel).end_reset_model();
    }
}

impl Session {
    fn section(&self) -> String {
        // XXX: stub
        "Section".into()
    }
}

define_model_roles! {
    enum SessionRoles for Session {
        ID(id):                                              "id",
        Source(source via QString::from):                    "source",
        IsGroup(is_group):                                   "isGroup",
        GroupName(group_name via qstring_from_option):       "groupName",
        GroupMembers(group_members via qstring_from_option): "groupMembers",
        Message(message via QString::from):                  "message",
        Section(fn section(&self) via QString::from):        "section",
        Timestamp(timestamp via qdatetime_from_i64):         "timestamp",
        Unread(unread):                                      "unread",
        Sent(sent):                                          "sent",
        Received(received):                                  "received",
        HasAttachment(has_attachment):                       "hasAttachment"
    }
}

impl QAbstractListModel for SessionModel {
    fn row_count(&self) -> i32 {
        self.content.len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let role = SessionRoles::from(role);
        role.get(&self.content[index.row() as usize])
    }

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        SessionRoles::role_names()
    }
}
