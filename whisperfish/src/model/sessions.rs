#![allow(non_snake_case)]

use crate::gui::AppState;
use crate::model::*;
use crate::store::orm;
use chrono::prelude::*;
use itertools::Itertools;
use qmeta_async::with_executor;
use qmetaobject::prelude::*;
use std::collections::HashMap;

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

/// QML-constructable object that interacts with a list of sessions.
///
/// Currently, this object will list all sessions unfiltered, ordered by the last message received
/// timestamp.
/// In the future, it should be possible to install filters and change the ordering.
#[derive(Default)]
pub struct SessionsImpl {
    session_list: QObjectBox<SessionListModel>,
}

crate::observing_model! {
    pub struct Sessions(SessionsImpl) {
        sessions: QVariant; READ sessions,
    }
}

impl SessionsImpl {
    fn init(&mut self, storage: Storage) {
        self.session_list.pinned().borrow_mut().load_all(storage);
    }

    fn sessions(&self) -> QVariant {
        self.session_list.pinned().into()
    }
}

impl EventObserving for SessionsImpl {
    fn observe(&mut self, storage: Storage, _event: crate::store::observer::Event) {
        self.session_list.pinned().borrow_mut().load_all(storage)
    }

    fn interests() -> Vec<crate::store::observer::Interest> {
        vec![crate::store::observer::Interest::All]
    }
}

#[derive(QObject, Default)]
pub struct SessionListModel {
    base: qt_base_class!(trait QAbstractListModel),
    content: Vec<AugmentedSession>,

    count: qt_method!(fn(&self) -> usize),
    unread: qt_method!(fn(&self) -> i32),
}

impl SessionListModel {
    fn load_all(&mut self, storage: Storage) {
        let sessions = storage.fetch_sessions().into_iter().map(|session| {
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

            let last_message =
                if let Some(last_message) = storage.fetch_last_message_by_session_id(session.id) {
                    last_message
                } else {
                    return (session, group_members, None);
                };
            let last_message_receipts = storage.fetch_message_receipts(last_message.id);
            let last_message_attachments = storage.fetch_attachments_for_message(last_message.id);

            (
                session,
                group_members,
                Some((
                    last_message,
                    last_message_attachments,
                    last_message_receipts,
                )),
            )
        });

        self.begin_reset_model();
        self.content = sessions
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

        self.end_reset_model();
    }

    fn count(&self) -> usize {
        self.content.len()
    }

    fn unread(&self) -> i32 {
        self.content
            .iter()
            .map(|session| if session.is_read() { 0 } else { 1 })
            .sum()
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

    fn group_description(&self) -> Option<String> {
        match &self.session.r#type {
            orm::SessionType::GroupV1(_) => None,
            orm::SessionType::GroupV2(group) => group.description.to_owned(),
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

    // FIXME we have them separated now... Get QML to understand it.
    fn group_member_names(&self) -> Option<String> {
        match &self.session.r#type {
            orm::SessionType::GroupV1(_group) => {
                Some(self.group_members.iter().map(|r| r.name()).join(","))
            }
            orm::SessionType::GroupV2(_group) => {
                Some(self.group_members.iter().map(|r| r.name()).join(","))
            }
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

    fn recipient_name(&self) -> &str {
        match &self.session.r#type {
            orm::SessionType::GroupV1(_group) => "",
            orm::SessionType::GroupV2(_group) => "",
            orm::SessionType::DirectMessage(recipient) => {
                recipient.profile_joined_name.as_deref().unwrap_or_default()
            }
        }
    }

    fn recipient_uuid(&self) -> &str {
        match &self.session.r#type {
            orm::SessionType::GroupV1(_group) => "",
            orm::SessionType::GroupV2(_group) => "",
            orm::SessionType::DirectMessage(recipient) => recipient.uuid(),
        }
    }

    fn recipient_emoji(&self) -> &str {
        match &self.session.r#type {
            orm::SessionType::GroupV1(_group) => "",
            orm::SessionType::GroupV2(_group) => "",
            orm::SessionType::DirectMessage(recipient) => {
                recipient.about_emoji.as_deref().unwrap_or_default()
            }
        }
    }

    fn has_avatar(&self) -> bool {
        match &self.session.r#type {
            orm::SessionType::GroupV1(_) => false,
            orm::SessionType::GroupV2(group) => group.avatar.is_some(),
            orm::SessionType::DirectMessage(recipient) => recipient.signal_profile_avatar.is_some(),
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
            .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
            .unwrap()
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
        !self.typing.is_empty()
    }

    // XXX exposing this as a model would be nicer, but it'll do for now.
    fn typing(&self) -> qmetaobject::QVariantList {
        let mut lst = qmetaobject::QVariantList::default();
        for t in &self.typing {
            lst.push(QString::from(format!("{}|{}", t.e164_or_uuid(), t.name())).into());
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
        RecipientName(fn recipient_name(&self) via QString::from):         "recipientName",
        RecipientUuid(fn recipient_uuid(&self) via QString::from):         "recipientUuid",
        RecipientEmoji(fn recipient_emoji(&self) via QString::from):       "recipientEmoji",
        IsGroup(fn is_group(&self)):                                       "isGroup",
        IsGroupV2(fn is_group_v2(&self)):                                  "isGroupV2",
        GroupId(fn group_id(&self) via qstring_from_option):               "groupId",
        GroupName(fn group_name(&self) via qstring_from_option):           "groupName",
        GroupDescription(fn group_description(&self) via qstring_from_option):
                                                                           "groupDescription",
        GroupMembers(fn group_members(&self) via qstring_from_option):     "groupMembers",
        GroupMemberNames(fn group_member_names(&self) via qstring_from_option):
                                                                           "groupMemberNames",
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
        HasAvatar(fn has_avatar(&self)):                                   "hasAvatar",

        IsTyping(fn is_typing(&self)):                                     "isTyping",
        Typing(fn typing(&self)):                                          "typing",
    }
}

impl QAbstractListModel for SessionListModel {
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
