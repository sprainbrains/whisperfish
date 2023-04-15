#![allow(non_snake_case)]

use crate::model::*;
use crate::schema;
use crate::store::observer::EventObserving;
use crate::store::observer::Interest;
use crate::store::orm;
use crate::store::Storage;
use qmetaobject::prelude::*;
use qmetaobject::QObjectBox;
use std::collections::HashMap;

/// QML-constructable object that interacts with a single session.
#[derive(Default, QObject)]
pub struct MessageImpl {
    base: qt_base_class!(trait QObject),
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
    type Context = ModelContext<Self>;

    fn observe(&mut self, ctx: Self::Context, _event: crate::store::observer::Event) {
        if let Some(id) = self.message_id {
            self.fetch(ctx.storage(), id);
        }
    }

    fn interests(&self) -> Vec<Interest> {
        self.message
            .iter()
            .flat_map(orm::AugmentedMessage::interests)
            .chain(self.message_id.iter().map(|mid| {
                Interest::whole_table_with_relation(
                    schema::attachments::table,
                    schema::messages::table,
                    *mid,
                )
            }))
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

    fn visual_attachments(&self) -> QVariant {
        self.visual_attachments.pinned().into()
    }

    fn fetch(&mut self, storage: Storage, id: i32) {
        self.message = storage.fetch_augmented_message(id);
        let attachments = storage.fetch_attachments_for_message(id);
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

    fn set_message_id(&mut self, ctx: Option<ModelContext<Self>>, id: i32) {
        if id >= 0 {
            self.message_id = Some(id);
            if let Some(ctx) = ctx {
                self.fetch(ctx.storage(), id);
            }
        } else {
            self.message_id = None;
            self.message = None;
            self.attachments.pinned().borrow_mut().set(Vec::new());
        }
    }

    fn init(&mut self, ctx: ModelContext<Self>) {
        if let Some(id) = self.message_id {
            self.fetch(ctx.storage(), id);
        }
    }
}

/// QML-constructable object that interacts with a single session.
#[derive(Default, QObject)]
pub struct SessionImpl {
    base: qt_base_class!(trait QObject),
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
        recipientId RecipientId,
        recipientName RecipientName,
        recipientUuid RecipientUuid,
        recipientE164 RecipientE164,
        recipientEmoji RecipientEmoji,
        recipientAboutText RecipientAbout,

        isGroup IsGroup,
        isGroupV2 IsGroupV2,
        isRegistered IsRegistered,

        groupId GroupId,
        groupName GroupName,
        groupDescription GroupDescription,

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
        draft Draft,

        typings Typings,
    }
}

impl EventObserving for SessionImpl {
    type Context = ModelContext<Self>;

    fn observe(&mut self, ctx: Self::Context, event: crate::store::observer::Event) {
        let storage = ctx.storage();
        if let Some(id) = self.session_id {
            let message_id = event
                .relation_key_for(schema::messages::table)
                .and_then(|x| x.as_i32());

            if event.for_row(schema::sessions::table, id) {
                self.session = storage.fetch_session_by_id_augmented(id);
                // XXX how to trigger a Qt signal now?
                return;
            } else if message_id.is_some() {
                self.session = storage.fetch_session_by_id_augmented(id);
                self.message_list
                    .pinned()
                    .borrow_mut()
                    .observe(storage, id, event);
                // XXX how to trigger a Qt signal now?
                return;
            }

            log::debug!(
                "Falling back to reloading the whole Session for event {:?}",
                event
            );
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

    fn set_session_id(&mut self, ctx: Option<ModelContext<Self>>, id: i32) {
        self.session_id = Some(id);
        if let Some(ctx) = ctx {
            self.fetch(ctx.storage(), id);
        }
    }

    fn init(&mut self, ctx: ModelContext<Self>) {
        if let Some(id) = self.session_id {
            self.fetch(ctx.storage(), id);
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

    fn observe(&mut self, storage: Storage, session_id: i32, event: crate::store::observer::Event) {
        // Waterfall handling of event.  If we cannot find a good specialized way of handling
        // the event, we'll reload the whole model.
        let message_id = event
            .relation_key_for(schema::messages::table)
            .and_then(|x| x.as_i32())
            .expect("message-related event observation");
        if event.is_delete() && event.for_table(schema::messages::table) {
            if let Some((pos, _msg)) = self
                .messages
                .iter()
                .enumerate()
                .find(|(_, msg)| msg.id == message_id)
            {
                self.begin_remove_rows(pos as i32, pos as i32);
                self.messages.remove(pos);
                self.end_remove_rows();
                return;
            }
        } else if event.is_update_or_insert() {
            let message = storage
                .fetch_augmented_message(message_id)
                .expect("inserted message");
            if message.session_id != session_id {
                log::trace!("Ignoring message insert/update for different session.");
                return;
            }
            let pos = self.messages.binary_search_by_key(
                &std::cmp::Reverse((message.server_timestamp, message.id)),
                |message| std::cmp::Reverse((message.server_timestamp, message.id)),
            );
            match pos {
                Ok(existing_index) => {
                    log::debug!("Handling update event.");
                    self.messages[existing_index] = message;
                    let idx = self.row_index(existing_index as i32);
                    self.data_changed(idx, idx);
                }
                Err(insertion_index) => {
                    log::debug!("Handling insertion event");
                    self.begin_insert_rows(insertion_index as i32, insertion_index as i32);
                    self.messages.insert(insertion_index, message);
                    self.end_insert_rows();
                }
            }
            return;
        }

        log::debug!(
            "Falling back to reloading the whole MessageListModel for event {:?}",
            event
        );
        self.load_all(storage, session_id);
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
