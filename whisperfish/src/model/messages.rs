#![allow(non_snake_case)]

use crate::model::*;
use crate::schema;
use crate::store::observer::EventObserving;
use crate::store::observer::Interest;
use crate::store::orm;
use crate::store::Storage;
use qmeta_async::with_executor;
use qmetaobject::prelude::*;
use qmetaobject::QObjectBox;
use std::collections::HashMap;

/// QML-constructable object that interacts with a single session.
#[derive(Default)]
pub struct MessageImpl {
    message_id: Option<i32>,
    message: Option<orm::AugmentedMessage>,

    attachments: QObjectBox<AttachmentListModel>,
    visual_attachments: QObjectBox<AttachmentListModel>,
    detail_attachments: QObjectBox<AttachmentListModel>,
}

crate::observing_model! {
    pub struct Message(MessageImpl) {
        messageId: i32; READ get_message_id WRITE set_message_id,
        valid: bool; READ get_valid,
        attachments: QVariant; READ attachments,
        thumbsAttachments: QVariant; READ visual_attachments,
        detailAttachments: QVariant; READ detail_attachments,

        thumbsAttachmentsCount: i32; READ visual_attachments_count,
        detailAttachmentsCount: i32; READ detail_attachments_count,
    } WITH OPTIONAL PROPERTIES FROM message WITH ROLE MessageRoles {
        sessionId SessionId,
        message Message,
        timestamp Timestamp,

        senderRecipientId SenderRecipientId,
        delivered Delivered,
        read Read,
        viewed Viewed,

        sent Sent,
        flags Flags,
        outgoing Outgoing,
        queued Queued,
        failed Failed,

        unidentifiedSender Unidentified,
        quotedMessageId QuotedMessageId,
    }
}

impl EventObserving for MessageImpl {
    fn observe(&mut self, storage: Storage, _event: crate::store::observer::Event) {
        if let Some(id) = self.message_id {
            self.fetch(storage, id);
        }
    }

    fn interests(&self) -> Vec<Interest> {
        self.message
            .iter()
            .flat_map(orm::AugmentedMessage::interests)
            .collect()
    }
}

impl MessageImpl {
    fn get_message_id(&self) -> i32 {
        self.message_id.unwrap_or(-1)
    }

    fn get_valid(&self) -> bool {
        self.message_id.is_some() && self.message.is_some()
    }

    fn attachments(&self) -> QVariant {
        self.attachments.pinned().into()
    }

    fn detail_attachments(&self) -> QVariant {
        self.detail_attachments.pinned().into()
    }

    fn detail_attachments_count(&self) -> i32 {
        self.detail_attachments.pinned().borrow().row_count()
    }

    fn visual_attachments(&self) -> QVariant {
        self.visual_attachments.pinned().into()
    }

    fn visual_attachments_count(&self) -> i32 {
        self.visual_attachments.pinned().borrow().row_count()
    }

    fn fetch(&mut self, storage: Storage, id: i32) {
        self.message = storage.fetch_augmented_message(id, true);
        let attachments = if let Some(message) = &self.message {
            message.attachments.clone()
        } else {
            Vec::new()
        };
        self.attachments
            .pinned()
            .borrow_mut()
            .set(attachments.clone());

        let (visual, detail) = attachments
            .into_iter()
            .partition(|x| x.content_type.contains("image") || x.content_type.contains("video"));

        self.detail_attachments.pinned().borrow_mut().set(detail);
        self.visual_attachments.pinned().borrow_mut().set(visual);
    }

    #[with_executor]
    fn set_message_id(&mut self, storage: Option<Storage>, id: i32) {
        if id >= 0 {
            self.message_id = Some(id);
            if let Some(storage) = storage {
                self.fetch(storage, id);
            }
        } else {
            self.message_id = None;
            self.message = None;
            self.attachments.pinned().borrow_mut().set(Vec::new());
        }
    }

    fn init(&mut self, storage: Storage) {
        if let Some(id) = self.message_id {
            self.fetch(storage, id);
        }
    }
}

/// QML-constructable object that interacts with a single session.
#[derive(Default)]
pub struct SessionImpl {
    session_id: Option<i32>,
    session: Option<orm::AugmentedSession>,
    message_list: QObjectBox<MessageListModel>,
}

crate::observing_model! {
    pub struct Session(SessionImpl) {
        sessionId: i32; READ get_session_id WRITE set_session_id,
        valid: bool; READ get_valid,
        messages: QVariant; READ messages,
    } WITH OPTIONAL PROPERTIES FROM session WITH ROLE SessionRoles {
        source Source,

        recipientName RecipientName,
        recipientUuid RecipientUuid,
        recipientE164 RecipientE164,
        recipientEmoji RecipientEmoji,
        recipientAboutText RecipientAbout,

        isGroup IsGroup,
        isGroupV2 IsGroupV2,

        groupId GroupId,
        groupName GroupName,
        groupDescription GroupDescription,
        groupMembers GroupMembers,
        groupMemberNames GroupMemberNames,

        message Message,
        section Section,
        timestamp Timestamp,
        read IsRead,
        sent Sent,
        deliveryCount Delivered,
        readCount Read,
        isMuted IsMuted,
        isArchived IsArchived,
        isPinned IsPinned,
        viewCount Viewed,
        hasAttachment HasAttachment,
        hasAvatar HasAvatar,
    }
}

impl EventObserving for SessionImpl {
    fn observe(&mut self, storage: Storage, _event: crate::store::observer::Event) {
        if let Some(id) = self.session_id {
            self.fetch(storage, id);
        }
    }

    fn interests(&self) -> Vec<Interest> {
        self.session
            .iter()
            .flat_map(orm::AugmentedSession::interests)
            .chain(
                self.message_list
                    .pinned()
                    .borrow()
                    .messages
                    .iter()
                    .flat_map(orm::AugmentedMessage::interests),
            )
            .chain(std::iter::once(Interest::whole_table(
                schema::messages::table,
            )))
            .collect()
    }
}

impl SessionImpl {
    fn get_session_id(&self) -> i32 {
        self.session_id.unwrap_or(-1)
    }

    fn get_valid(&self) -> bool {
        self.session_id.is_some() && self.session.is_some()
    }

    fn fetch(&mut self, storage: Storage, id: i32) {
        self.session = storage.fetch_session_by_id_augmented(id);
        self.message_list
            .pinned()
            .borrow_mut()
            .load_all(storage, id);
    }

    #[with_executor]
    fn set_session_id(&mut self, storage: Option<Storage>, id: i32) {
        self.session_id = Some(id);
        if let Some(storage) = storage {
            self.fetch(storage, id);
        }
    }

    fn init(&mut self, storage: Storage) {
        if let Some(id) = self.session_id {
            self.fetch(storage, id);
        }
    }

    fn messages(&self) -> QVariant {
        self.message_list.pinned().into()
    }
}

define_model_roles! {
    enum MessageRoles for orm::AugmentedMessage {
        Id(id):                                               "id",
        SessionId(session_id):                                "sessionId",
        Message(text via qstring_from_option):                "message",
        Timestamp(server_timestamp via qdatetime_from_naive): "timestamp",

        SenderRecipientId(sender_recipient_id via qvariant_from_option): "senderRecipientId",

        Delivered(fn delivered(&self)):                       "delivered",
        Read(fn read(&self)):                                 "read",
        Viewed(fn viewed(&self)):                             "viewed",

        Sent(fn sent(&self)):                                 "sent",
        Flags(flags):                                         "flags",
        Outgoing(is_outbound):                                "outgoing",
        Queued(fn queued(&self)):                             "queued",
        Failed(sending_has_failed):                           "failed",

        Attachments(fn attachments(&self)): "attachments",

        Unidentified(use_unidentified):                       "unidentifiedSender",
        QuotedMessageId(quote_id via qvariant_from_option):   "quotedMessageId",
    }
}

#[derive(QObject, Default)]
pub struct MessageListModel {
    base: qt_base_class!(trait QAbstractListModel),
    messages: Vec<orm::AugmentedMessage>,
}

impl MessageListModel {
    fn load_all(&mut self, storage: Storage, id: i32) {
        self.begin_reset_model();
        self.messages = storage
            .fetch_all_messages_augmented(id)
            .into_iter()
            .map(Into::into)
            .collect();
        self.end_reset_model();
    }
}

impl QAbstractListModel for MessageListModel {
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
