#![allow(non_snake_case)]

use std::collections::HashMap;
use std::collections::HashSet;

use crate::actor;
use crate::model::*;
use crate::store::orm;

use actix::prelude::*;
use chrono::prelude::*;
use futures::prelude::*;
use itertools::Itertools;
use qmeta_async::with_executor;
use qmetaobject::prelude::*;

// XXX attachments and receipts could be a compressed form.
struct AugmentedSession {
    session: orm::Session,
    group_members: Vec<orm::Recipient>,
    last_message: Option<LastMessage>,

    typing: Vec<orm::Recipient>,
}

// XXX Maybe make this AugmentedMessage
// use crate::store::orm::AugmentedMessage;
struct LastMessage {
    message: orm::Message,
    attachments: Vec<orm::Attachment>,
    receipts: Vec<(orm::Receipt, orm::Recipient)>,
}

impl std::ops::Deref for AugmentedSession {
    type Target = orm::Session;

    fn deref(&self) -> &orm::Session {
        &self.session
    }
}

#[derive(QObject, Default)]
pub struct SessionModel {
    base: qt_base_class!(trait QAbstractListModel),
    pub actor: Option<Addr<actor::SessionActor>>,

    content: Vec<AugmentedSession>,

    unread: qt_property!(i32; NOTIFY unread_count_changed READ unread),

    unread_count_changed: qt_signal!(),

    count: qt_method!(fn(&self) -> usize),
    add: qt_method!(fn(&self, id: i32, mark_read: bool)),
    remove: qt_method!(fn(&self, idx: usize)),
    removeById: qt_method!(fn(&self, id: i32)),
    reload: qt_method!(fn(&self)),

    markRead: qt_method!(fn(&mut self, id: usize)),
    markReceived: qt_method!(fn(&self, id: usize)),
    markSent: qt_method!(fn(&self, id: usize, message: QString)),
    markMuted: qt_method!(fn(&self, idx: usize, muted: bool)),
    markArchived: qt_method!(fn(&self, idx: usize, archived: bool)),
    markPinned: qt_method!(fn(&self, idx: usize, pinned: bool)),

    removeIdentities: qt_method!(fn(&self, session_id: i32)),
}

impl SessionModel {
    #[with_executor]
    fn count(&self) -> usize {
        self.content.len()
    }

    fn unread(&self) -> i32 {
        self.content
            .iter()
            .map(|session| if session.is_read() { 0 } else { 1 })
            .sum()
    }

    /// Add or replace a Session in the model.
    #[with_executor]
    fn add(&self, id: i32, mark_read: bool) {
        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::FetchSession { id, mark_read })
                .map(Result::unwrap),
        );
        log::trace!("Dispatched actor::FetchSession({})", id);
    }

    /// Removes session at index. This removes the session from the list model and
    /// deletes it from the database.
    #[with_executor]
    fn remove(&mut self, idx: usize) {
        if idx > self.content.len() - 1 {
            log::error!("Invalid index for session model");
            return;
        }

        let sid = self.content[idx].id;

        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::DeleteSession { id: sid, idx })
                .map(Result::unwrap),
        );
        log::trace!("Dispatched actor::DeleteSession({})", idx);
    }

    /// Removes session by id. This removes the session from the list model and
    /// deletes it from the database.
    #[with_executor]
    fn removeById(&self, id: i32) {
        let idx = self
            .content
            .iter()
            .position(|x| x.id == id)
            .expect("Session ID not found in session model");

        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::DeleteSession { id, idx })
                .map(Result::unwrap),
        );
        log::trace!("Dispatched actor::DeleteSession({})", idx);
    }

    #[with_executor]
    fn reload(&self) {
        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::LoadAllSessions)
                .map(Result::unwrap),
        );
    }

    #[with_executor]
    fn markRead(&mut self, id: usize) {
        if let Some((idx, session)) = self
            .content
            .iter_mut()
            .enumerate()
            .find(|(_, s)| s.id == id as i32)
        {
            actix::spawn(
                self.actor
                    .as_ref()
                    .unwrap()
                    .send(actor::MarkSessionRead {
                        sid: session.id,
                        already_unread: !session.is_read(),
                    })
                    .map(Result::unwrap),
            );

            if let Some(message) = &mut session.last_message {
                message.message.is_read = true;
            }
            let idx = (self as &mut dyn QAbstractListModel).row_index(idx as i32);
            (self as &mut dyn QAbstractListModel).data_changed(idx, idx);
        }

        self.unread_count_changed();

        // XXX: don't forget sync messages
    }

    #[with_executor]
    fn markReceived(&mut self, id: usize) {
        log::trace!("Mark received called");
        // XXX: don't forget sync messages
        if let Some((idx, session)) = self
            .content
            .iter_mut()
            .enumerate()
            .find(|(_, s)| s.id == id as i32)
        {
            if let Some(message) = &mut session.last_message {
                message.message.is_read = true;
            }
            let idx = (self as &mut dyn QAbstractListModel).row_index(idx as i32);
            (self as &mut dyn QAbstractListModel).data_changed(idx, idx);
        }

        self.unread_count_changed();
    }

    #[with_executor]
    fn markSent(&self, _id: usize, _message: QString) {
        log::trace!("STUB: Mark sent called");
        // XXX: don't forget sync messages
    }

    #[with_executor]
    fn markMuted(&self, idx: usize, muted: bool) {
        if idx > self.content.len() - 1 {
            log::error!("Invalid index for session model");
            return;
        }

        let sid = self.content[idx].id;

        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::MarkSessionMuted { sid, muted })
                .map(Result::unwrap),
        );
        log::trace!("Dispatched actor::MarkSessionMuted({}, {})", idx, muted);
    }

    #[with_executor]
    fn markArchived(&self, idx: usize, archived: bool) {
        if idx > self.content.len() - 1 {
            log::error!("Invalid index for session model");
            return;
        }

        let sid = self.content[idx].id;

        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::MarkSessionArchived { sid, archived })
                .map(Result::unwrap),
        );
        log::trace!(
            "Dispatched actor::MarkSessionArchived({}, {})",
            idx,
            archived
        );
    }

    #[with_executor]
    fn markPinned(&self, idx: usize, pinned: bool) {
        if idx > self.content.len() - 1 {
            log::error!("Invalid index for session model");
            return;
        }

        let sid = self.content[idx].id;

        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::MarkSessionPinned { sid, pinned })
                .map(Result::unwrap),
        );
        log::trace!("Dispatched actor::MarkSessionPinned({}, {})", idx, pinned);
    }

    #[with_executor]
    fn removeIdentities(&self, session_id: i32) {
        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::RemoveIdentities { session_id })
                .map(Result::unwrap),
        );
        log::trace!("Dispatched SessionActor::RemoveIdentities({})", session_id);
    }

    // Event handlers below this line

    /// Handle loaded session
    ///
    /// The session is accompanied by the last message that was sent on this session.
    #[allow(clippy::type_complexity)]
    pub fn handle_sessions_loaded(
        &mut self,
        sessions: Vec<(
            orm::Session,
            Vec<orm::Recipient>,
            Option<(
                orm::Message,
                Vec<orm::Attachment>,
                Vec<(orm::Receipt, orm::Recipient)>,
            )>,
        )>,
    ) {
        // XXX: maybe this should be called before even accessing the db?
        (self as &mut dyn QAbstractListModel).begin_reset_model();
        self.content = sessions
            .into_iter()
            .map(|(session, group_members, last_message)| AugmentedSession {
                session,
                group_members,
                last_message: last_message.map(|(message, attachments, receipts)| LastMessage {
                    message,
                    attachments,
                    receipts,
                }),
                // XXX migrate typing notices from previous loaded sessions?
                typing: Vec::new(),
            })
            .collect();
        // XXX This could be solved through a sub query.
        self.content
            .sort_unstable_by(|a, b| match (&a.last_message, &b.last_message) {
                (Some(a_last_message), Some(b_last_message)) => b_last_message
                    .message
                    .server_timestamp
                    .cmp(&a_last_message.message.server_timestamp),
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(_), None) => std::cmp::Ordering::Greater,
                // Gotta use something here.
                (None, None) => a.session.id.cmp(&b.session.id),
            });
        // Stable sort, such that this retains the above ordering.
        self.content.sort_by_key(|k| !k.is_pinned);
        (self as &mut dyn QAbstractListModel).end_reset_model();

        self.unread_count_changed();
    }

    /// Handle add-or-replace session
    pub fn handle_fetch_session(
        &mut self,
        sess: orm::Session,
        group_members: Vec<orm::Recipient>,
        mut last_message: orm::Message,
        last_message_attachments: Vec<orm::Attachment>,
        last_message_receipts: Vec<(orm::Receipt, orm::Recipient)>,
        mark_read: bool,
    ) {
        let sid = sess.id;
        let mut already_unread = false;

        let found = self
            .content
            .iter()
            .enumerate()
            .find(|(_i, s)| s.id == sess.id);

        if let Some((idx, _session)) = found {
            already_unread = !last_message.is_read;

            log::trace!("Removing the session from qml");

            // Remove from this place so it can be added back in later
            (self as &mut dyn QAbstractListModel).begin_remove_rows(idx as i32, idx as i32);
            self.content.remove(idx);
            (self as &mut dyn QAbstractListModel).end_remove_rows();
        };

        if !last_message.is_read && mark_read {
            actix::spawn(
                self.actor
                    .as_ref()
                    .unwrap()
                    .send(actor::MarkSessionRead {
                        sid: sess.id,
                        already_unread,
                    })
                    .map(Result::unwrap),
            );
            log::trace!(
                "Dispatched actor::MarkSessionRead({}, {})",
                sid,
                already_unread
            );
            last_message.is_read = true;

        // unimplemented!();
        } else if last_message.is_read && !already_unread {
            // TODO: model.session.go:181
            // let count = self.unread() + 1;

            // self.set_unread(count);
            // self.unread_changed(count);
        }

        let mut newIdx = 0_usize;

        for (idx, s) in self.content.iter().enumerate() {
            if s.is_pinned == sess.is_pinned {
                newIdx = idx;
                break;
            }
        }

        log::trace!("Inserting the session back in qml into position {}", newIdx);

        (self as &mut dyn QAbstractListModel).begin_insert_rows(newIdx as i32, newIdx as i32);
        self.content.insert(
            newIdx,
            AugmentedSession {
                session: sess,
                group_members,
                last_message: Some(LastMessage {
                    message: last_message,
                    attachments: last_message_attachments,
                    receipts: last_message_receipts,
                }),
                typing: Vec::new(),
            },
        );
        (self as &mut dyn QAbstractListModel).end_insert_rows();

        self.unread_count_changed();
    }

    /// Handle add-or-replace session (without is_read update)
    pub fn handle_update_session(
        &mut self,
        sess: orm::Session,
        group_members: Vec<orm::Recipient>,
        last_message: orm::Message,
        last_message_attachments: Vec<orm::Attachment>,
        last_message_receipts: Vec<(orm::Receipt, orm::Recipient)>,
    ) {
        let found = self
            .content
            .iter()
            .enumerate()
            .find(|(_i, s)| s.id == sess.id);

        let typing = if let Some((idx, _session)) = found {
            log::trace!("Removing the session from qml");

            // Remove from this place so it can be added back in later
            (self as &mut dyn QAbstractListModel).begin_remove_rows(idx as i32, idx as i32);
            let typing = std::mem::take(&mut self.content[idx].typing);
            self.content.remove(idx);
            (self as &mut dyn QAbstractListModel).end_remove_rows();
            typing
        } else {
            Vec::new()
        };

        let mut newIdx = 0_usize;

        for (idx, s) in self.content.iter().enumerate() {
            if s.is_pinned == sess.is_pinned {
                newIdx = idx;
                break;
            }
        }

        log::trace!("Inserting the session back in qml into position {}", newIdx);

        (self as &mut dyn QAbstractListModel).begin_insert_rows(newIdx as i32, newIdx as i32);
        self.content.insert(
            newIdx,
            AugmentedSession {
                session: sess,
                group_members,
                last_message: Some(LastMessage {
                    message: last_message,
                    attachments: last_message_attachments,
                    receipts: last_message_receipts,
                }),
                typing,
            },
        );
        (self as &mut dyn QAbstractListModel).end_insert_rows();

        self.unread_count_changed();
    }

    /// The typings should map session id's to recipients.
    pub fn handle_update_typing(&mut self, mut typings: HashMap<i32, Vec<orm::Recipient>>) {
        // Cannot iter_mut() over content with .enumerate(), because we need to call data_changed()
        // too.
        for i in 0..self.content.len() {
            let session = &mut self.content[i];
            let changed = if let Some(typers) = typings.remove(&session.id) {
                let lhs: HashSet<i32> = session.typing.iter().map(|s| s.id).collect();
                let rhs: HashSet<i32> = typers.iter().map(|s| s.id).collect();

                session.typing = typers;
                lhs != rhs
            } else if !session.typing.is_empty() {
                log::trace!("Clearing typing for session id {} at row {}", session.id, i);
                session.typing.clear();
                true
            } else {
                false
            };
            if changed {
                log::trace!("Updating model for changed typings");
                let idx = self.row_index(i as i32);
                self.data_changed(idx, idx);
            }
        }

        if !typings.is_empty() {
            log::warn!(
                "Some session is typing that this model does not now about: {:?}",
                typings
            )
        }
    }

    /// When a session is marked as read and this handler called, implicitly
    /// the session will be set at the top of the QML list.
    pub fn handle_mark_session_read(&mut self, sid: i32, already_unread: bool) {
        if let Some((i, session)) = self
            .content
            .iter_mut()
            .enumerate()
            .find(|(_, s)| s.session.id == sid)
        {
            if let Some(m) = &mut session.last_message {
                m.message.is_read = true;
            }
            let idx = (self as &mut dyn QAbstractListModel).row_index(i as i32);
            (self as &mut dyn QAbstractListModel).data_changed(idx, idx);
        } else {
            log::warn!("Could not call data_changed for non-existing session!");
        }

        if already_unread {
            // TODO: model.session.go:173
            // let count = std::cmp::min(0, self.unread() - 1);

            // self.set_unread(count);
            // self.unread_changed(count);
        }

        self.unread_count_changed();
    }

    pub fn handle_mark_session_muted(&mut self, sid: i32, muted: bool) {
        if let Some((i, session)) = self
            .content
            .iter_mut()
            .enumerate()
            .find(|(_, s)| s.session.id == sid)
        {
            session.session.is_muted = muted;
            let idx = (self as &mut dyn QAbstractListModel).row_index(i as i32);
            (self as &mut dyn QAbstractListModel).data_changed(idx, idx);
        } else {
            log::warn!("Could not call data_changed for non-existing session!");
        }
    }

    pub fn handle_mark_session_archived(&mut self, sid: i32, archived: bool) {
        if let Some((i, session)) = self
            .content
            .iter_mut()
            .enumerate()
            .find(|(_, s)| s.session.id == sid)
        {
            session.session.is_archived = archived;
            let idx = (self as &mut dyn QAbstractListModel).row_index(i as i32);
            (self as &mut dyn QAbstractListModel).data_changed(idx, idx);
        } else {
            log::warn!("Could not call data_changed for non-existing session!");
        }
    }

    pub fn handle_mark_session_pinned(&mut self, sid: i32, pinned: bool) {
        if let Some((_i, session)) = self
            .content
            .iter_mut()
            .enumerate()
            .find(|(_, s)| s.session.id == sid)
        {
            session.session.is_pinned = pinned;

            actix::spawn(
                self.actor
                    .as_ref()
                    .unwrap()
                    .send(actor::UpdateSession {
                        id: session.session.id,
                    })
                    .map(Result::unwrap),
            );
            log::trace!("Dispatched actor::UpdateSession({})", session.session.id);
        } else {
            log::warn!("Could not call data_changed for non-existing session!");
        }
    }

    /// Remove deleted session from QML
    pub fn handle_delete_session(&mut self, idx: usize) {
        (self as &mut dyn QAbstractListModel).begin_remove_rows(idx as i32, idx as i32);
        self.content.remove(idx);
        (self as &mut dyn QAbstractListModel).end_remove_rows();

        self.unread_count_changed();
    }
}

impl AugmentedSession {
    fn timestamp(&self) -> Option<NaiveDateTime> {
        self.last_message
            .as_ref()
            .map(|m| m.message.server_timestamp)
    }

    fn group_name(&self) -> Option<&str> {
        match &self.session.r#type {
            orm::SessionType::GroupV1(group) => Some(&group.name),
            orm::SessionType::GroupV2(group) => Some(&group.name),
            orm::SessionType::DirectMessage(_) => None,
        }
    }

    fn group_id(&self) -> Option<&str> {
        match &self.session.r#type {
            orm::SessionType::GroupV1(group) => Some(&group.id),
            orm::SessionType::GroupV2(group) => Some(&group.id),
            orm::SessionType::DirectMessage(_) => None,
        }
    }

    // FIXME we have them separated now... Get QML to understand it.
    fn group_members(&self) -> Option<String> {
        match &self.session.r#type {
            orm::SessionType::GroupV1(_group) => Some(
                self.group_members
                    .iter()
                    .map(|r| r.e164_or_uuid())
                    .join(","),
            ),
            orm::SessionType::GroupV2(_group) => Some(
                self.group_members
                    .iter()
                    .map(|r| r.e164_or_uuid())
                    .join(","),
            ),
            orm::SessionType::DirectMessage(_) => None,
        }
    }

    fn sent(&self) -> bool {
        if let Some(m) = &self.last_message {
            m.message.sent_timestamp.is_some()
        } else {
            false
        }
    }

    fn source(&self) -> &str {
        match &self.session.r#type {
            orm::SessionType::GroupV1(_group) => "",
            orm::SessionType::GroupV2(_group) => "",
            orm::SessionType::DirectMessage(recipient) => recipient.e164_or_uuid(),
        }
    }

    fn has_attachment(&self) -> bool {
        if let Some(m) = &self.last_message {
            !m.attachments.is_empty()
        } else {
            false
        }
    }

    fn text(&self) -> Option<&str> {
        self.last_message
            .as_ref()
            .and_then(|m| m.message.text.as_deref())
    }

    fn section(&self) -> String {
        if self.session.is_pinned {
            return String::from("pinned");
        }

        // XXX: stub
        let now = chrono::Utc::now();
        let today = Utc
            .ymd(now.year(), now.month(), now.day())
            .and_hms(0, 0, 0)
            .naive_utc();

        let last_message = if let Some(m) = &self.last_message {
            &m.message
        } else {
            return String::from("today");
        };
        let diff = today.signed_duration_since(last_message.server_timestamp);

        if diff.num_seconds() <= 0 {
            String::from("today")
        } else if diff.num_hours() <= 24 {
            String::from("yesterday")
        } else if diff.num_hours() <= (7 * 24) {
            let wd = last_message.server_timestamp.weekday().number_from_monday() % 7;
            wd.to_string()
        } else {
            String::from("older")
        }
    }

    fn is_read(&self) -> bool {
        self.last_message
            .as_ref()
            .map(|m| m.message.is_read)
            .unwrap_or(false)
    }

    fn delivered(&self) -> u32 {
        if let Some(m) = &self.last_message {
            m.receipts
                .iter()
                .filter(|(r, _)| r.delivered.is_some())
                .count() as _
        } else {
            0
        }
    }

    fn read(&self) -> u32 {
        if let Some(m) = &self.last_message {
            m.receipts.iter().filter(|(r, _)| r.read.is_some()).count() as _
        } else {
            0
        }
    }

    fn is_muted(&self) -> bool {
        self.session.is_muted
    }

    fn is_archived(&self) -> bool {
        self.session.is_archived
    }

    fn is_pinned(&self) -> bool {
        self.session.is_pinned
    }

    fn viewed(&self) -> u32 {
        if let Some(m) = &self.last_message {
            m.receipts
                .iter()
                .filter(|(r, _)| r.viewed.is_some())
                .count() as _
        } else {
            0
        }
    }

    fn is_typing(&self) -> bool {
        log::trace!("QML request is_typing");
        !self.typing.is_empty()
    }

    // XXX exposing this as a model would be nicer, but it'll do for now.
    fn typing(&self) -> qmetaobject::QVariantList {
        log::trace!("QML request typing");
        let mut lst = qmetaobject::QVariantList::default();
        for t in &self.typing {
            lst.push(QString::from(t.e164_or_uuid()).into());
        }
        lst
    }
}

define_model_roles! {
    // FIXME: many of these are now functions because of backwards compatibility.
    //        swap them around for real fields, and fixup QML instead.
    enum SessionRoles for AugmentedSession {
        Id(id):                                                            "id",
        Source(fn source(&self) via QString::from):                        "source",
        IsGroup(fn is_group(&self)):                                       "isGroup",
        IsGroupV2(fn is_group_v2(&self)):                                  "isGroupV2",
        GroupId(fn group_id(&self) via qstring_from_option):               "groupId",
        GroupName(fn group_name(&self) via qstring_from_option):           "groupName",
        GroupMembers(fn group_members(&self) via qstring_from_option):     "groupMembers",
        Message(fn text(&self) via qstring_from_option):                   "message",
        Section(fn section(&self) via QString::from):                      "section",
        Timestamp(fn timestamp(&self) via qdatetime_from_naive_option):    "timestamp",
        IsRead(fn is_read(&self)):                                         "read",
        Sent(fn sent(&self)):                                              "sent",
        Delivered(fn delivered(&self)):                                    "deliveryCount",
        Read(fn read(&self)):                                              "readCount",
        IsMuted(fn is_muted(&self)):                                       "isMuted",
        IsArchived(fn is_archived(&self)):                                 "isArchived",
        IsPinned(fn is_pinned(&self)):                                     "isPinned",
        Viewed(fn viewed(&self)):                                          "viewCount",
        HasAttachment(fn has_attachment(&self)):                           "hasAttachment",

        IsTyping(fn is_typing(&self)):                                     "isTyping",
        Typing(fn typing(&self)):                                          "typing",
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
