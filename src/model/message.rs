#![allow(non_snake_case)]

use std::collections::HashMap;
use std::ops::Deref;
use std::process::Command;

use crate::actor;
use crate::model::*;
use crate::store::orm::Receipt;
use crate::store::orm::Recipient;
use crate::store::orm::{self, Attachment, AugmentedMessage};
use crate::worker::{ClientActor, SendMessage};

use actix::prelude::*;
use futures::prelude::*;
use itertools::Itertools;
use qmeta_async::with_executor;
use qmetaobject::prelude::*;
use qmetaobject::{QObjectBox, QObjectPinned};

define_model_roles! {
    enum MessageRoles for QtAugmentedMessage {
        Id(id):                                               "id",
        Sid(session_id):                                      "sid",
        Source(fn source(&self) via QString::from):           "source",
        Message(text via qstring_from_option):                "message",
        Timestamp(server_timestamp via qdatetime_from_naive): "timestamp",

        Delivered(fn delivered(&self)):                       "delivered",
        Read(fn read(&self)):                                 "read",
        Viewed(fn viewed(&self)):                             "viewed",

        Reactions(fn reactions(&self) via QString::from):     "reactions",

        Sent(fn sent(&self)):                                 "sent",
        Flags(flags):                                         "flags",
        ThumbsAttachments(fn visual_attachments(&self)):      "thumbsAttachments",
        DetailAttachments(fn detail_attachments(&self)):      "detailAttachments",
        Outgoing(is_outbound):                                "outgoing",
        Queued(fn queued(&self)):                             "queued",
        Failed(sending_has_failed):                           "failed",

        // FIXME issue #11 multiple attachments
        Attachment(fn first_attachment(&self) via QString::from): "attachment",
        AttachmentMimeType(fn first_attachment_mime_type(&self) via QString::from): "mimeType",
    }
}

struct QtAugmentedMessage {
    inner: AugmentedMessage,
    visual_attachments: QObjectBox<AttachmentModel>,
    detail_attachments: QObjectBox<AttachmentModel>,
}

impl Deref for QtAugmentedMessage {
    type Target = AugmentedMessage;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<AugmentedMessage> for QtAugmentedMessage {
    fn from(inner: AugmentedMessage) -> Self {
        let (visual, detail) =
            inner.attachments.iter().cloned().partition(|x| {
                x.content_type.contains("image") || x.content_type.contains("video")
            });

        let visual_attachments = AttachmentModel {
            attachments: visual,
            ..Default::default()
        };
        let detail_attachments = AttachmentModel {
            attachments: detail,
            ..Default::default()
        };
        Self {
            inner,
            visual_attachments: QObjectBox::new(visual_attachments),
            detail_attachments: QObjectBox::new(detail_attachments),
        }
    }
}

#[derive(QObject, Default)]
pub struct MessageModel {
    base: qt_base_class!(trait QAbstractListModel),
    pub actor: Option<Addr<actor::MessageActor>>,
    pub client_actor: Option<Addr<ClientActor>>,

    messages: Vec<QtAugmentedMessage>,

    group_members: Vec<orm::Recipient>,
    fingerprint: Option<String>,

    sessionId: qt_property!(i32; NOTIFY sessionIdChanged),

    numericFingerprint: qt_property!(QString; NOTIFY peerIdentityChanged READ fingerprint),
    peerName: qt_property!(QString; NOTIFY peerChanged),
    peerTel: qt_property!(QString; NOTIFY peerChanged),
    peerUuid: qt_property!(QString; NOTIFY peerChanged),

    groupMembers: qt_property!(QString; NOTIFY groupMembersChanged),
    groupId: qt_property!(QString; NOTIFY groupChanged),
    group: qt_property!(bool; NOTIFY groupChanged),
    groupV1: qt_property!(bool; NOTIFY groupChanged),
    groupV2: qt_property!(bool; NOTIFY groupChanged),

    peerIdentityChanged: qt_signal!(),
    peerChanged: qt_signal!(),
    groupMembersChanged: qt_signal!(),
    sessionIdChanged: qt_signal!(),
    groupChanged: qt_signal!(),

    openAttachment: qt_method!(fn(&self, index: usize)),
    createGroupMessage: qt_method!(
        fn(
            &self,
            group_id: QString,
            message: QString,
            groupName: QString,
            attachment: QString,
            add: bool,
        ) -> i32
    ),
    createMessage: qt_method!(
        fn(&self, source: QString, message: QString, attachment: QString, add: bool) -> i32
    ),

    sendMessage: qt_method!(fn(&self, mid: i32)),
    endSession: qt_method!(fn(&self, e164: QString)),

    load: qt_method!(fn(&self, sid: i32, peer_name: QString)),
    reload_message: qt_method!(fn(&self, mid: i32)),
    add: qt_method!(fn(&self, id: i32)),
    remove: qt_method!(
        fn(
            &self,
            id: usize, /* FIXME the implemented method takes an *index* but should take a message ID */
        )
    ),

    markSent: qt_method!(fn(&self, id: i32)),
    markReceived: qt_method!(fn(&self, id: i32)),
    markFailed: qt_method!(fn(&self, id: i32)),
    markPending: qt_method!(fn(&self, id: i32)),
}

impl MessageModel {
    #[with_executor]
    fn openAttachment(&mut self, idx: usize) {
        let msg = if let Some(msg) = self.messages.get(idx) {
            msg
        } else {
            log::error!("[attachment] Message not found at index {}", idx);
            return;
        };

        // XXX move this method to its own model.
        let attachment = msg.first_attachment();

        log::debug!("[attachment] Open by index {:?}: {}", idx, &attachment);

        match Command::new("xdg-open").arg(attachment).status() {
            Ok(status) => {
                if !status.success() {
                    log::error!("[attachment] fail");
                }
            }
            Err(e) => {
                log::error!("[attachment] Error {}", e);
            }
        }
    }

    #[with_executor]
    fn createGroupMessage(
        &mut self,
        group_id: QString,
        message: QString,
        _group_name: QString,
        attachment: QString,
        _add: bool,
    ) -> i32 {
        let group_id = group_id.to_string();
        let message = message.to_string();
        let attachment = attachment.to_string();

        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                // XXX hackermannnnnn
                .send(match group_id.len() {
                    32 => actor::QueueGroupMessage::GroupV1Message {
                        group_id,
                        message,
                        attachment,
                    },
                    64 => actor::QueueGroupMessage::GroupV2Message {
                        group_id,
                        message,
                        attachment,
                    },
                    _ => unreachable!("Illegal group ID"),
                })
                .map(Result::unwrap),
        );

        // TODO: QML should *not* synchronously wait for a session ID to be returned.
        -1
    }

    #[with_executor]
    fn createMessage(
        &mut self,
        e164: QString,
        message: QString,
        attachment: QString,
        _add: bool,
    ) -> i32 {
        let e164 = e164.to_string();
        let message = message.to_string();
        let attachment = attachment.to_string();

        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::QueueMessage {
                    e164,
                    message,
                    attachment,
                })
                .map(Result::unwrap),
        );

        // TODO: QML should *not* synchronously wait for a session ID to be returned.
        -1
    }

    /// Called when a message should be queued to be sent to OWS
    #[with_executor]
    fn sendMessage(&mut self, mid: i32) {
        actix::spawn(
            self.client_actor
                .as_mut()
                .unwrap()
                .send(SendMessage(mid))
                .map(Result::unwrap),
        );
    }

    /// Called when a message should be queued to be sent to OWS
    #[with_executor]
    fn endSession(&mut self, e164: QString) {
        actix::spawn(
            self.actor
                .as_mut()
                .unwrap()
                .send(actor::EndSession(e164.into()))
                .map(Result::unwrap),
        );
    }

    pub fn handle_queue_message(&mut self, msg: orm::Message, attachments: Vec<orm::Attachment>) {
        self.sendMessage(msg.id);

        // TODO: Go version modified the `self` model appropriately,
        //       with the `add`/`_add` parameter from createMessage.
        // if add {
        (self as &mut dyn QAbstractListModel).begin_insert_rows(0, 0);
        self.messages.insert(
            0,
            AugmentedMessage {
                inner: msg,
                sender: None,
                // XXX
                attachments,
                // No receipts yet.
                receipts: Vec::new(),
                // No reactions yet.
                reactions: Vec::new(),
            }
            .into(),
        );
        (self as &mut dyn QAbstractListModel).end_insert_rows();
        // }
    }

    #[with_executor]
    fn reload_message(&mut self, mid: i32) {
        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::FetchMessage(mid))
                .map(Result::unwrap),
        );
        log::trace!("Dispatched actor::FetchMessage({})", mid);
    }

    #[with_executor]
    fn load(&mut self, sid: i32, _peer_name: QString) {
        (self as &mut dyn QAbstractListModel).begin_reset_model();

        self.messages.clear();

        (self as &mut dyn QAbstractListModel).end_reset_model();

        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::FetchSession {
                    id: sid,
                    mark_read: false,
                })
                .map(Result::unwrap),
        );
        log::trace!("Dispatched actor::FetchSession({})", sid);
    }

    /// Adds a message to QML list.
    ///
    /// This retrieves a `Message` by the given id and adds it to the UI.
    ///
    /// Note that the id argument was i64 in Go.
    #[with_executor]
    fn add(&mut self, id: i32) {
        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::FetchMessage(id))
                .map(Result::unwrap),
        );
        log::trace!("Dispatched actor::FetchMessage({})", id);
    }

    /// Remove a message from both QML and database
    ///
    /// Note the Go code said main thread only. This is
    /// satisfied in Rust by sending the request to the
    /// main thread.
    ///
    /// FIXME Take a message ID instead of an index.
    #[with_executor]
    pub fn remove(&self, idx: usize) {
        let msg = if let Some(msg) = self.messages.get(idx) {
            msg
        } else {
            log::error!("[remove] Message not found at index {}", idx);
            return;
        };

        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::DeleteMessage(msg.id, idx))
                .map(Result::unwrap),
        );

        log::trace!("Dispatched actor::DeleteMessage({}, {})", msg.id, idx);
    }

    #[with_executor]
    fn fingerprint(&self) -> QString {
        self.fingerprint
            .as_deref()
            .unwrap_or("no fingerprint")
            .into()
    }

    /// Mark a message sent in QML.
    ///
    /// Called through QML. Maybe QML doesn't know how
    /// to pass booleans, because this and `mark_received`
    /// simply wrap the real workhorse.
    ///
    /// Note that the id argument was i64 in Go.
    #[with_executor]
    fn markSent(&mut self, id: i32) {
        self.mark(id, true, false, false, false)
    }

    /// Mark a message received in QML.
    ///
    /// Called through QML. Maybe QML doesn't know how
    /// to pass booleans, because this and `mark_sent`
    /// simply wrap the real workhorse.
    ///
    /// Note that the id argument was i64 in Go.
    #[with_executor]
    fn markReceived(&mut self, id: i32) {
        self.mark(id, false, true, false, false)
    }

    /// Mark a message failed
    #[with_executor]
    fn markFailed(&mut self, id: i32) {
        self.mark(id, false, false, true, false)
    }

    /// Mark a message pending/queued
    #[with_executor]
    fn markPending(&mut self, id: i32) {
        self.mark(id, false, false, false, true)
    }

    /// Mark a message sent or received in QML. No database involved.
    ///
    /// Note that the id argument was i64 in Go.
    #[with_executor]
    fn mark(
        &mut self,
        id: i32,
        mark_sent: bool,
        mark_received: bool,
        mark_failed: bool,
        mark_pending: bool,
    ) {
        if mark_sent && mark_received {
            log::trace!("Cannot mark message both sent and received");
            return;
        }

        if let Some((i, mut msg)) = self
            .messages
            .iter_mut()
            .enumerate()
            .find(|(_, msg)| msg.id == id)
        {
            if mark_sent {
                log::trace!("Mark message {} sent '{}'", id, mark_sent);

                // XXX: fetch the correct time
                msg.inner.inner.sent_timestamp = Some(chrono::Utc::now().naive_utc());
            } else if mark_failed {
                log::trace!("Mark message {} failed'", id);
                msg.inner.inner.sending_has_failed = true;
            } else if mark_pending {
                log::trace!("Mark message {} failed'", id);
                msg.inner.inner.sending_has_failed = false;
            } else if mark_received {
                log::trace!("Mark message {} received '{}'", id, mark_received);

                // XXX: fetch the correct time and person
                msg.inner.inner.received_timestamp = Some(chrono::Utc::now().naive_utc());
                // Dummy
                msg.inner.receipts.push((
                    Receipt {
                        message_id: 0,
                        recipient_id: 0,
                        delivered: msg.inner.received_timestamp,
                        read: None,
                        viewed: None,
                    },
                    Recipient {
                        id,
                        e164: None,
                        uuid: None,
                        username: None,
                        email: None,
                        blocked: false,
                        profile_key: None,
                        profile_key_credential: None,
                        profile_given_name: None,
                        profile_family_name: None,
                        profile_joined_name: None,
                        signal_profile_avatar: None,
                        profile_sharing: false,
                        last_profile_fetch: None,
                        unidentified_access_mode: false,
                        storage_service_id: None,
                        storage_proto: None,
                        capabilities: 0,
                        last_gv1_migrate_reminder: None,
                        last_session_reset: None,
                    },
                ));
            }
            // In fact, we should only update the necessary roles, but qmetaobject, in its current
            // state, does not allow this.
            // , MessageRoles::Received);
            // We'll also have troubles with the mutable borrow over `msg`, but that's nothing we
            // cannot solve.  We're saved by NLL here.
            let idx = (self as &mut dyn QAbstractListModel).row_index(i as i32);
            (self as &mut dyn QAbstractListModel).data_changed(idx, idx);
        } else {
            log::error!("Message not found");
        }
    }

    // Event handlers below this line

    /// Handle a fetched session from message's point of view
    pub fn handle_fetch_session(
        &mut self,
        sess: orm::Session,
        group_members: Vec<orm::Recipient>,
        fingerprint: Option<String>,
    ) {
        log::trace!("handle_fetch_session({})", sess.id);
        self.sessionId = sess.id;
        self.sessionIdChanged();

        self.group_members = group_members;

        match sess.r#type {
            orm::SessionType::GroupV1(group) => {
                self.peerTel = QString::from("");
                self.peerUuid = QString::from("");
                self.peerName = QString::from(group.name.deref());
                self.peerChanged();

                self.group = true;
                self.groupV1 = true;
                self.groupV2 = false;
                self.groupId = QString::from(group.id);
                self.groupChanged();

                self.groupMembers = QString::from(
                    self.group_members
                        .iter()
                        .map(|r| r.e164_or_uuid())
                        .join(","),
                );
                self.groupMembersChanged();
            }
            orm::SessionType::GroupV2(group) => {
                self.peerTel = QString::from("");
                self.peerUuid = QString::from("");
                self.peerName = QString::from(group.name.deref());
                self.peerChanged();

                self.group = true;
                self.groupV1 = false;
                self.groupV2 = true;
                self.groupId = QString::from(group.id);
                self.groupChanged();

                self.groupMembers = QString::from(
                    self.group_members
                        .iter()
                        .map(|r| r.e164_or_uuid())
                        .join(","),
                );
                self.groupMembersChanged();
            }
            orm::SessionType::DirectMessage(recipient) => {
                self.group = false;
                self.groupV1 = false;
                self.groupV2 = false;
                self.groupId = QString::from("");
                self.groupChanged();

                self.peerTel = QString::from(recipient.e164.as_deref().unwrap_or(""));
                self.peerUuid = QString::from(recipient.uuid.as_deref().unwrap_or(""));
                self.peerName = QString::from(recipient.e164_or_uuid());
                self.peerChanged();
            }
        };

        self.fingerprint = fingerprint;
        self.peerIdentityChanged();

        // TODO: contact identity key
        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::FetchAllMessages(sess.id))
                .map(Result::unwrap),
        );
        log::trace!("Dispatched actor::FetchAllMessages({})", sess.id);
    }

    pub fn handle_fetch_message(
        &mut self,
        message: orm::Message,
        recipient: Option<orm::Recipient>,
        attachments: Vec<orm::Attachment>,
        receipts: Vec<(orm::Receipt, orm::Recipient)>,
        reactions: Vec<(orm::Reaction, orm::Recipient)>,
    ) {
        log::trace!("handle_fetch_message({})", message.id);

        let idx = self.messages.binary_search_by(|am| {
            am.inner
                .server_timestamp
                .cmp(&message.server_timestamp)
                .reverse()
        });

        let am = AugmentedMessage {
            inner: message,
            sender: recipient,
            attachments,
            receipts,
            reactions,
        }
        .into();

        match idx {
            Ok(idx) => {
                log::trace!("Fetched message exists at idx = {}; replacing", idx);
                let model_idx = (self as &mut dyn QAbstractListModel).row_index(idx as _);

                self.messages[idx] = am;
                (self as &mut dyn QAbstractListModel).data_changed(model_idx, model_idx);
            }
            Err(idx) => {
                (self as &mut dyn QAbstractListModel).begin_insert_rows(idx as _, idx as _);
                self.messages.insert(idx, am);
                (self as &mut dyn QAbstractListModel).end_insert_rows();
            }
        };
    }

    #[allow(clippy::type_complexity)]
    pub fn handle_fetch_all_messages(&mut self, messages: Vec<orm::AugmentedMessage>) {
        log::trace!(
            "handle_fetch_all_messages({}) count {}",
            // XXX What if no messages?
            messages[0].session_id,
            messages.len()
        );

        (self as &mut dyn QAbstractListModel).begin_insert_rows(0, messages.len() as i32);

        self.messages.extend(messages.into_iter().map(Into::into));

        (self as &mut dyn QAbstractListModel).end_insert_rows();
    }

    pub fn handle_delete_message(&mut self, id: i32, idx: usize, del_rows: usize) {
        log::trace!(
            "handle_delete_message({}) deleted {} rows, remove qml idx {}",
            id,
            del_rows,
            idx
        );

        (self as &mut dyn QAbstractListModel).begin_remove_rows(idx as i32, idx as i32);

        self.messages.remove(idx);

        (self as &mut dyn QAbstractListModel).end_remove_rows();
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

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        MessageRoles::role_names()
    }
}

define_model_roles! {
    enum AttachmentRoles for Attachment {
        // There's a lot more useful stuff to expose.
        MimeType(content_type via QString::from):       "type",
        Data(attachment_path via qstring_from_option):  "data",
    }
}

#[derive(QObject, Default)]
pub struct AttachmentModel {
    base: qt_base_class!(trait QAbstractListModel),
    attachments: Vec<Attachment>,

    count: qt_property!(i32; NOTIFY rowCountChanged READ row_count),

    /// Gets the nth item of the model, serialized as byte array
    get: qt_method!(fn(&self, idx: i32) -> QByteArray),

    rowCountChanged: qt_signal!(),
}

impl AttachmentModel {
    // XXX When we're able to run Rust 1.a-bit-more, with qmetaobject 0.2.7+, we have QVariantMap.
    fn get(&self, idx: i32) -> QByteArray {
        let mut map = qmetaobject::QJsonObject::default();

        for (k, v) in self.role_names() {
            let idx = self.row_index(idx);
            map.insert(
                v.to_str().expect("only utf8 role names"),
                self.data(idx, k).into(),
            );
        }

        map.to_json()
    }
}

impl QtAugmentedMessage {
    fn detail_attachments(&self) -> QObjectPinned<'_, AttachmentModel> {
        self.detail_attachments.pinned()
    }

    fn visual_attachments(&self) -> QObjectPinned<'_, AttachmentModel> {
        self.visual_attachments.pinned()
    }
}

impl QAbstractListModel for AttachmentModel {
    fn row_count(&self) -> i32 {
        self.attachments.len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let role = AttachmentRoles::from(role);
        role.get(&self.attachments[index.row() as usize])
    }

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        AttachmentRoles::role_names()
    }
}
