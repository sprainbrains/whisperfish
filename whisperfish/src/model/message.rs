#![allow(non_snake_case)]

use crate::gui::AppState;
use crate::model::*;
use crate::store::orm::{Attachment, AugmentedMessage};
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
