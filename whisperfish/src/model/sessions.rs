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
#[derive(Default, QObject)]
pub struct SessionsImpl {
    base: qt_base_class!(trait QObject),
    session_list: QObjectBox<SessionListModel>,
}

crate::observing_model! {
    pub struct Sessions(SessionsImpl) {
        sessions: QVariant; READ sessions,

        count: usize; READ count,
        unread: usize; READ unread,
    }
}

impl SessionsImpl {
    fn init(&mut self, storage: Storage) {
        self.session_list.pinned().borrow_mut().load_all(storage);
    }

    fn sessions(&self) -> QVariant {
        self.session_list.pinned().into()
    }

    fn count(&self) -> usize {
        self.session_list.pinned().borrow().count()
    }

    fn unread(&self) -> usize {
        self.session_list.pinned().borrow().unread()
    }
}

impl EventObserving for SessionsImpl {
    fn observe(&mut self, storage: Storage, event: crate::store::observer::Event) {
        // Find the correct session and update the latest message
        let session_id = event
            .relation_key_for(schema::sessions::table)
            .and_then(|x| x.as_i32());
        let message_id = event
            .relation_key_for(schema::messages::table)
            .and_then(|x| x.as_i32());
        if session_id.is_some() || message_id.is_some() {
            self.session_list
                .pinned()
                .borrow_mut()
                .observe(storage, event);
            return;
        }

        log::trace!(
            "Falling back to reloading the whole Sessions model for event {:?}",
            event
        );
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

    count: qt_property!(usize; READ count NOTIFY countChanged),
    unread: qt_property!(usize; READ unread NOTIFY countChanged),

    countChanged: qt_signal!(),
}

impl SessionListModel {
    fn load_all(&mut self, storage: Storage) {
        self.begin_reset_model();
        self.content = storage.fetch_all_sessions_augmented();

        // Stable sort, such that this retains the above ordering.
        self.content.sort_by_key(|k| !k.is_pinned);
        self.end_reset_model();
        self.countChanged();
    }

    fn observe(&mut self, storage: Storage, event: crate::store::observer::Event) {
        let session_id = event
            .relation_key_for(schema::sessions::table)
            .and_then(|x| x.as_i32());
        let message_id = event
            .relation_key_for(schema::messages::table)
            .and_then(|x| x.as_i32());

        if let Some(session_id) = session_id {
            // Remove session from the model if exists
            let idx = self
                .content
                .iter()
                .enumerate()
                .find(|(_, s)| s.id == session_id);
            if let Some((idx, _session)) = idx {
                self.begin_remove_rows(idx as i32, idx as i32);
                self.content.remove(idx);
                self.end_remove_rows();
                self.countChanged();
            }

            if let Some(session) = storage.fetch_session_by_id_augmented(session_id) {
                let idx = self
                    .content
                    .binary_search_by_key(
                        &std::cmp::Reverse((
                            session.last_message.as_ref().map(|m| &m.server_timestamp),
                            session.id,
                        )),
                        |session| {
                            std::cmp::Reverse((
                                session.last_message.as_ref().map(|m| &m.server_timestamp),
                                session.id,
                            ))
                        },
                    )
                    .expect_err("removed session");
                self.begin_insert_rows(idx as i32, idx as i32);
                self.content.insert(idx, session);
                self.end_insert_rows();
                self.countChanged();
            } else {
                assert!(event.for_table(schema::sessions::table));
                assert!(event.is_delete());
            }
        } else if let Some(message_id) = message_id {
            // There's no relation to a session, so that means that an augmented message was
            // updated.
            let mut range = None;
            for (idx, session) in self.content.iter_mut().enumerate() {
                if let Some(message) = &mut session.last_message {
                    if message.id == message_id {
                        // XXX This can in principle fetch a message with another timestamp,
                        // but I think all those cases are handled with a session_id
                        session.last_message =
                            storage.fetch_last_message_by_session_id_augmented(session.id);
                        let (low, high) = range.get_or_insert((idx, idx));
                        if *low > idx {
                            *low = idx;
                        }
                        if *high < idx {
                            *high = idx;
                        }
                    }
                }
            }

            if let Some((low, high)) = range {
                let low = self.row_index(low as i32);
                let high = self.row_index(high as i32);
                self.data_changed(low, high);
            }
        } else {
            log::warn!("Unimplemented: Sessions model observe without message_id or session_id");
        }
    }

    fn count(&self) -> usize {
        self.content.len()
    }

    fn unread(&self) -> usize {
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
