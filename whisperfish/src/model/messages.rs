#![allow(non_snake_case)]

use super::attachment::*;
use crate::gui::AppState;
use crate::model::*;
use crate::store::observer::EventObserving;
use crate::store::orm::AugmentedMessage;
use qmeta_async::with_executor;
use qmetaobject::prelude::*;
use qmetaobject::{QObjectBox, QObjectPinned};
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

/// QML-constructable object that interacts with a single session.
#[derive(QObject, Default)]
pub struct Session {
    base: qt_base_class!(trait QObject),

    app: qt_property!(QPointer<AppState>; WRITE set_app),
    _sessionId: qt_property!(i32; WRITE set_session_id READ get_session_id ALIAS sessionId),
    messages: qt_property!(QVariant; READ messages CONST),

    message_list: ObservingModel<MessageListModel>,
}

impl Session {
    #[with_executor]
    fn set_app(&mut self, app: QPointer<AppState>) {
        self.app = app;
        self.reinit();
    }

    fn get_session_id(&mut self) -> i32 {
        self.message_list.pinned().borrow().session_id.unwrap_or(-1)
    }

    #[with_executor]
    fn set_session_id(&mut self, id: i32) {
        self.message_list.pinned().borrow_mut().session_id = Some(id);
        self.reinit();
    }

    fn reinit(&mut self) {
        if let Some(app) = self.app.as_pinned() {
            if let Some(storage) = app.borrow().storage.borrow().clone() {
                self.message_list.register(storage.clone());
                if self.message_list.pinned().borrow().session_id.is_some() {
                    self.message_list.pinned().borrow_mut().load_all(storage);
                }
            }
        }
    }

    fn messages(&mut self) -> QVariant {
        self.message_list.pinned().into()
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

        let visual_attachments = AttachmentModel::new(visual);
        let detail_attachments = AttachmentModel::new(detail);

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
pub struct MessageListModel {
    base: qt_base_class!(trait QAbstractListModel),

    session_id: Option<i32>,
    messages: Vec<QtAugmentedMessage>,
}

impl EventObserving for MessageListModel {
    fn observe(&mut self, storage: Storage, _event: crate::store::observer::Event) {
        self.load_all(storage);
    }

    fn interests() -> Vec<crate::store::observer::Interest> {
        vec![crate::store::observer::Interest::All]
    }
}

impl MessageListModel {
    fn load_all(&mut self, storage: Storage) {
        self.messages = storage
            .fetch_all_messages_augmented(self.session_id.expect("session_id set when loading"))
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
