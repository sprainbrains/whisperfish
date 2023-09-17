#![allow(non_snake_case)]

use crate::model::*;
use crate::store::observer::{EventObserving, Interest};
use crate::store::{orm, schema, Storage};
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
        messageId: i32;              READ get_message_id     WRITE set_message_id NOTIFY message_id_changed,
        valid: bool;                 READ get_valid                               NOTIFY valid_changed,
        attachments: QVariant;       READ attachments                             NOTIFY attachments_changed,
        thumbsAttachments: QVariant; READ visual_attachments                      NOTIFY thumb_attachments_changed,
        detailAttachments: QVariant; READ detail_attachments                      NOTIFY detail_attachments_changed,
    } WITH OPTIONAL PROPERTIES FROM message WITH ROLE MessageRoles {
        sessionId: QVariant;          ROLE SessionId         READ get_sessionId           NOTIFY sessionId_changed,
        message: QVariant;            ROLE Message           READ get_message             NOTIFY message_changed,
        timestamp: QVariant;          ROLE Timestamp         READ get_timestamp           NOTIFY timestamp_changed,

        senderRecipientId: QVariant;  ROLE SenderRecipientId READ get_sender_recipient_id NOTIFY sender_recipient_id_changed,
        delivered: QVariant;          ROLE Delivered         READ get_delivered           NOTIFY delivered_changed,
        read: QVariant;               ROLE Read              READ get_read                NOTIFY read_changed,
        viewed: QVariant;             ROLE Viewed            READ get_viewed              NOTIFY viewed_changed,

        sent: QVariant;               ROLE Sent              READ get_sent                NOTIFY sent_changed,
        flags: QVariant;              ROLE Flags             READ get_flags               NOTIFY flags_changed,
        outgoing: QVariant;           ROLE Outgoing          READ get_outgoing            NOTIFY outgoing_changed,
        queued: QVariant;             ROLE Queued            READ get_queued              NOTIFY queued_changed,
        failed: QVariant;             ROLE Failed            READ get_failed              NOTIFY failed_changed,
        remoteDeleted: QVariant;      ROLE RemoteDeleted     READ get_remote_deleted      NOTIFY remote_deleted_changed,

        unidentifiedSender: QVariant; ROLE Unidentified      READ get_unidentified_sender NOTIFY unidentified_sender_changed,
        quotedMessageId: QVariant;    ROLE QuotedMessageId   READ get_quoted_message_id   NOTIFY quoted_message_id_changed,
    }
}

impl EventObserving for MessageImpl {
    type Context = ModelContext<Self>;

    fn observe(&mut self, ctx: Self::Context, event: crate::store::observer::Event) {
        if let Some(id) = self.message_id {
            if let Some(attachment_id) = event.relation_key_for(schema::attachments::table) {
                if event.is_delete() {
                    // XXX This could also be implemented efficiently
                    self.fetch(ctx.storage(), id);
                } else {
                    // Only reload the attachments.
                    // We could also just reload the necessary attachment, but we're lazy today.
                    self.load_attachment(ctx.storage(), id, attachment_id.as_i32().unwrap());
                }
            } else {
                self.fetch(ctx.storage(), id);
            }
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
        self.fetch_attachments(storage, id);
    }

    fn fetch_attachments(&mut self, storage: Storage, id: i32) {
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

    fn load_attachment(&mut self, storage: Storage, _id: i32, attachment_id: i32) {
        let attachment = storage
            .fetch_attachment(attachment_id)
            .expect("existing attachment");

        for container in &[
            &self.attachments,
            if attachment.content_type.contains("image")
                || attachment.content_type.contains("video")
            {
                &self.visual_attachments
            } else {
                &self.detail_attachments
            },
        ] {
            container
                .pinned()
                .borrow_mut()
                .update_attachment(attachment.clone());
        }
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
        sessionId: i32;     READ get_session_id WRITE set_session_id NOTIFY session_id_changed,
        valid: bool;        READ get_valid                           NOTIFY valid_changed,
        messages: QVariant; READ messages                            NOTIFY messages_changed,
    } WITH OPTIONAL PROPERTIES FROM session WITH ROLE SessionRoles {
        recipientId: QVariant;        ROLE RecipientId      READ get_recipient_id         NOTIFY recipient_id_changed,
        recipientName: QVariant;      ROLE RecipientName    READ get_recipient_name       NOTIFY recipient_name_changed,
        recipientUuid: QVariant;      ROLE RecipientUuid    READ get_recipient_uuid       NOTIFY recipient_uuid_changed,
        recipientE164: QVariant;      ROLE RecipientE164    READ get_recipient_e164       NOTIFY recipient_e164_changed,
        recipientEmoji: QVariant;     ROLE RecipientEmoji   READ get_recipient_emoji      NOTIFY recipient_emoji_changed,
        recipientAboutText: QVariant; ROLE RecipientAbout   READ get_recipient_about_text NOTIFY recipient_about_text_changed,

        isGroup: QVariant;            ROLE IsGroup          READ get_is_group             NOTIFY is_group_changed,
        isGroupV2: QVariant;          ROLE IsGroupV2        READ get_is_group_v2          NOTIFY is_group_v2_changed,
        isRegistered: QVariant;       ROLE IsRegistered     READ get_is_registered        NOTIFY is_registered_changed,

        groupId: QVariant;            ROLE GroupId          READ get_group_id             NOTIFY group_id_changed,
        groupName: QVariant;          ROLE GroupName        READ get_group_name           NOTIFY group_name_changed,
        groupDescription: QVariant;   ROLE GroupDescription READ get_group_description    NOTIFY group_description_changed,

        message: QVariant;            ROLE Message          READ get_message              NOTIFY message_changed,
        section: QVariant;            ROLE Section          READ get_section              NOTIFY section_changed,
        timestamp: QVariant;          ROLE Timestamp        READ get_timestamp            NOTIFY timestamp_changed,
        read: QVariant;               ROLE IsRead           READ get_read                 NOTIFY read_changed,
        sent: QVariant;               ROLE Sent             READ get_sent                 NOTIFY sent_changed,
        deliveryCount: QVariant;      ROLE Delivered        READ get_delivery_count       NOTIFY delivery_count_changed,
        readCount: QVariant;          ROLE Read             READ get_read_count           NOTIFY read_count_changed,
        isMuted: QVariant;            ROLE IsMuted          READ get_is_muted             NOTIFY is_muted_changed,
        isArchived: QVariant;         ROLE IsArchived       READ get_is_archived          NOTIFY is_archived_changed,
        isPinned: QVariant;           ROLE IsPinned         READ get_is_pinned            NOTIFY is_pinned_changed,
        viewCount: QVariant;          ROLE Viewed           READ get_view_count           NOTIFY view_count_changed,
        hasAttachment: QVariant;      ROLE HasAttachment    READ get_has_attachment       NOTIFY has_attachment_changed,
        hasAvatar: QVariant;          ROLE HasAvatar        READ get_has_avatar           NOTIFY has_avatar_changed,
        draft: QVariant;              ROLE Draft            READ get_draft                NOTIFY draft_changed,
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

            if event.for_table(schema::attachments::table) && event.is_update() {
                // Don't care, because AugmentedMessage only takes into account the number of
                // attachments.
                return;
            }

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
        RemoteDeleted(is_remote_deleted):                     "remoteDeleted",

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
