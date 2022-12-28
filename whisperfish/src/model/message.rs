#![allow(non_snake_case)]

use crate::actor;
use crate::gui::AppState;
use crate::model::*;
use crate::store::orm::{Attachment, AugmentedMessage};
use crate::worker::{ClientActor, SendMessage};
use actix::prelude::*;
use futures::prelude::*;
use qmeta_async::with_executor;
use qmetaobject::prelude::*;
use qmetaobject::{QObjectBox, QObjectPinned};
use std::collections::HashMap;
use std::ops::Deref;
use std::process::Command;
use std::rc::Rc;

/// QML-constructable object that interacts with a single session.
#[derive(QObject, Default)]
pub struct Session {
    base: qt_base_class!(trait QObject),

    app: qt_property!(std::cell::RefCell<AppState>; WRITE set_app),
    sessionId: qt_property!(i32; WRITE set_session_id),
}

impl Session {
    fn set_app(&mut self, app: std::cell::RefCell<AppState>) {
        self.app = app;
        self.reinit();
    }

    fn set_session_id(&mut self, id: i32) {
        self.sessionId = id;
        self.reinit();
    }

    fn reinit(&mut self) {}
}

impl Drop for Session {
    fn drop(&mut self) {
        // TODO deregister interest in sessions table
    }
}

define_model_roles! {
    enum MessageRoles for QtAugmentedMessage {
        Id(id):                                               "id",
        Sid(session_id):                                      "sid",
        Source(fn source(&self) via QString::from):           "source",
        PeerName(fn peerName(&self) via QString::from):       "peerName",
        Message(text via qstring_from_option):                "message",
        Timestamp(server_timestamp via qdatetime_from_naive): "timestamp",

        Delivered(fn delivered(&self)):                       "delivered",
        Read(fn read(&self)):                                 "read",
        Viewed(fn viewed(&self)):                             "viewed",

        Reactions(fn reactions(&self) via QString::from):     "reactions",
        ReactionsFull(fn reactions_full(&self) via QString::from):
                                                              "reactionsNamed",

        Sent(fn sent(&self)):                                 "sent",
        Flags(flags):                                         "flags",
        ThumbsAttachments(fn visual_attachments(&self)):      "thumbsAttachments",
        DetailAttachments(fn detail_attachments(&self)):      "detailAttachments",
        Outgoing(is_outbound):                                "outgoing",
        Queued(fn queued(&self)):                             "queued",
        Failed(sending_has_failed):                           "failed",

        Unidentified(use_unidentified):                       "unidentifiedSender",
        QuotedMessage(fn quote(&self)):                       "quote",
    }
}

#[derive(Clone, Default)]
struct QtAugmentedMessage {
    inner: AugmentedMessage,
    visual_attachments: Rc<QObjectBox<AttachmentModel>>,
    detail_attachments: Rc<QObjectBox<AttachmentModel>>,

    quoted_message: Option<Box<QtAugmentedMessage>>,
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

        let quoted_message = inner.quoted_message.clone().map(|x| Box::new((*x).into()));
        Self {
            inner,
            visual_attachments: Rc::new(QObjectBox::new(visual_attachments)),
            detail_attachments: Rc::new(QObjectBox::new(detail_attachments)),
            quoted_message,
        }
    }
}

#[derive(QObject, Default)]
pub struct MessageModel {
    base: qt_base_class!(trait QAbstractListModel),

    messages: Vec<QtAugmentedMessage>,
}

#[derive(QObject, Default)]
pub struct MessageMethods {
    base: qt_base_class!(trait QObject),
    pub actor: Option<Addr<actor::MessageActor>>,
    pub client_actor: Option<Addr<ClientActor>>,

    // XXX move into Session
    fingerprint: Option<String>,

    numericFingerprint: qt_property!(QString; NOTIFY peerIdentityChanged READ fingerprint),
    peerName: qt_property!(QString; NOTIFY peerChanged),
    peerTel: qt_property!(QString; NOTIFY peerChanged),
    peerUuid: qt_property!(QString; NOTIFY peerChanged),
    peerHasAvatar: qt_property!(bool; NOTIFY peerChanged),
    aboutEmoji: qt_property!(QString; NOTIFY peerChanged),
    aboutText: qt_property!(QString; NOTIFY peerChanged),

    groupMembers: qt_property!(QString; NOTIFY groupMembersChanged),
    groupMemberNames: qt_property!(QString; NOTIFY groupMembersChanged),
    groupMemberUuids: qt_property!(QString; NOTIFY groupMembersChanged),
    groupId: qt_property!(QString; NOTIFY groupChanged),
    group: qt_property!(bool; NOTIFY groupChanged),
    groupV1: qt_property!(bool; NOTIFY groupChanged),
    groupV2: qt_property!(bool; NOTIFY groupChanged),
    groupDescription: qt_property!(QString; NOTIFY peerChanged),

    peerIdentityChanged: qt_signal!(),
    peerChanged: qt_signal!(),
    groupMembersChanged: qt_signal!(),
    sessionIdChanged: qt_signal!(),
    groupChanged: qt_signal!(),

    createMessage: qt_method!(
        fn(
            &self,
            session_id: i32,
            message: QString,
            attachment: QString,
            quote: i32,
            add: bool,
        ) -> i32
    ),

    sendMessage: qt_method!(fn(&self, mid: i32)),
    endSession: qt_method!(fn(&self, e164: QString)),

    remove: qt_method!(
        fn(
            &self,
            id: i32, /* FIXME the implemented method takes an *index* but should take a message ID */
        )
    ),
}

impl Drop for MessageMethods {
    fn drop(&mut self) {
        todo!()
    }
}

impl MessageMethods {
    #[with_executor]
    fn createMessage(
        &mut self,
        session_id: i32,
        message: QString,
        attachment: QString,
        quote: i32,
        _add: bool,
    ) -> i32 {
        let message = message.to_string();
        let attachment = attachment.to_string();

        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::QueueMessage {
                    session_id,
                    message,
                    attachment,
                    quote,
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

    /// Remove a message from the database.
    #[with_executor]
    pub fn remove(&self, id: i32) {
        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::DeleteMessage(id))
                .map(Result::unwrap),
        );

        log::trace!("Dispatched actor::DeleteMessage({})", id);
    }

    #[with_executor]
    fn fingerprint(&self) -> QString {
        self.fingerprint
            .as_deref()
            .unwrap_or("no fingerprint")
            .into()
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

    open: qt_method!(fn(&self, idx: i32)),

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

    #[with_executor]
    fn open(&mut self, idx: i32) {
        let attachment = if let Some(attachment) = self.attachments.get(idx as usize) {
            attachment
        } else {
            log::error!("[attachment] Message not found at index {}", idx);
            return;
        };
        let attachment = if let Some(path) = &attachment.attachment_path {
            path
        } else {
            log::error!("[attachment] Opening attachment without path (idx {})", idx);
            return;
        };

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
}

impl QtAugmentedMessage {
    fn detail_attachments(&self) -> QObjectPinned<'_, AttachmentModel> {
        self.detail_attachments.pinned()
    }

    fn visual_attachments(&self) -> QObjectPinned<'_, AttachmentModel> {
        self.visual_attachments.pinned()
    }

    // XXX When we're able to run Rust 1.a-bit-more, with qmetaobject 0.2.7+, we have QVariantMap.
    fn quote(&self) -> QVariant {
        if let Some(quote) = &self.quoted_message {
            let mut map = qmetaobject::QJsonObject::default();

            for (k, v) in MessageRoles::role_names() {
                map.insert(
                    v.to_str().expect("only utf8 role names"),
                    MessageRoles::from(k).get(quote).into(),
                );
            }
            QVariant::from(map.to_json())
        } else {
            QVariant::default()
        }
    }

    fn peerName(&self) -> &str {
        match &self.sender {
            Some(s) => s.profile_joined_name.as_deref().unwrap_or_default(),
            None => "",
        }
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
