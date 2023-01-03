#![allow(non_snake_case)]

use crate::model::*;
use crate::store::observer::EventObserving;
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
}

crate::observing_model! {
    pub struct Message(MessageImpl) {
        messageId: i32; READ get_message_id WRITE set_message_id,
        attachments: QVariant; READ attachments,
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

    fn interests() -> Vec<crate::store::observer::Interest> {
        vec![crate::store::observer::Interest::All]
    }
}

impl MessageImpl {
    fn get_message_id(&self) -> i32 {
        self.message_id.unwrap_or(-1)
    }

    fn attachments(&self) -> QVariant {
        self.attachments.pinned().into()
    }

    fn fetch(&mut self, storage: Storage, id: i32) {
        self.message = storage.fetch_augmented_message(id, true);
        let attachments = if let Some(message) = &self.message {
            message.attachments.clone()
        } else {
            Vec::new()
        };
        self.attachments.pinned().borrow_mut().set(attachments);
    }

    #[with_executor]
    fn set_message_id(&mut self, storage: Option<Storage>, id: i32) {
        self.message_id = Some(id);
        if let Some(storage) = storage {
            self.fetch(storage, id);
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
        messages: QVariant; READ messages,
    } WITH OPTIONAL PROPERTIES FROM session WITH ROLE SessionRoles {
        source Source,

        recipientName RecipientName,
        recipientUuid RecipientUuid,
        recipientEmoji RecipientEmoji,

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

    fn interests() -> Vec<crate::store::observer::Interest> {
        vec![crate::store::observer::Interest::All]
    }
}

impl SessionImpl {
    fn get_session_id(&self) -> i32 {
        self.session_id.unwrap_or(-1)
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
        self.messages = storage
            .fetch_all_messages_augmented(id)
            .into_iter()
            .map(Into::into)
            .collect();
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
