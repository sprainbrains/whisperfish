#![allow(non_snake_case)]

use crate::store::observer::{EventObserving, Interest};
use crate::store::{orm, Storage};
use crate::{model::*, schema};
use qmetaobject::prelude::*;
use std::collections::HashMap;

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

        count: i32; READ count,
        unread: i32; READ unread,
    }
}

impl SessionsImpl {
    fn init(&mut self, storage: Storage) {
        self.session_list.pinned().borrow_mut().load_all(storage);
    }

    fn sessions(&self) -> QVariant {
        self.session_list.pinned().into()
    }

    fn count(&self) -> i32 {
        self.session_list.pinned().borrow().count() as _
    }

    fn unread(&self) -> i32 {
        self.session_list.pinned().borrow().unread()
    }
}

impl EventObserving for SessionsImpl {
    fn observe(&mut self, storage: Storage, event: crate::store::observer::Event) {
        if event.for_table(schema::reactions::table) {
            return;
        }
        if event.for_table(schema::messages::table) {
            // Find the correct session and update the latest message
            // XXX this required the session id; relations are currently not part of an event
            // return;
        }
        self.session_list.pinned().borrow_mut().load_all(storage)
    }

    fn interests(&self) -> Vec<crate::store::observer::Interest> {
        std::iter::once(Interest::whole_table(schema::sessions::table))
            .chain(
                self.session_list
                    .pinned()
                    .borrow()
                    .content
                    .iter()
                    .flat_map(|session| session.interests()),
            )
            .collect()
    }
}

#[derive(QObject, Default)]
pub struct SessionListModel {
    base: qt_base_class!(trait QAbstractListModel),
    content: Vec<orm::AugmentedSession>,

    count: qt_method!(fn(&self) -> usize),
    unread: qt_method!(fn(&self) -> i32),
}

impl SessionListModel {
    fn load_all(&mut self, storage: Storage) {
        self.begin_reset_model();
        self.content = storage.fetch_all_sessions_augmented();

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

define_model_roles! {
    // FIXME: many of these are now functions because of backwards compatibility.
    //        swap them around for real fields, and fixup QML instead.
    pub(super) enum SessionRoles for orm::AugmentedSession {
        Id(id):                                                            "id",
        Source(fn source(&self) via QString::from):                        "source",
        RecipientName(fn recipient_name(&self) via QString::from):         "recipientName",
        RecipientUuid(fn recipient_uuid(&self) via QString::from):         "recipientUuid",
        RecipientE164(fn recipient_e164(&self) via QString::from):         "recipientE164",
        RecipientEmoji(fn recipient_emoji(&self) via QString::from):       "recipientEmoji",
        RecipientAbout(fn recipient_about(&self) via QString::from):       "recipientAboutText",
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
