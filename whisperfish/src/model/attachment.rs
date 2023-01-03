#![allow(non_snake_case)]

use crate::model::*;
use crate::store::orm;
use std::collections::HashMap;
use std::process::Command;

define_model_roles! {
    enum AttachmentRoles for orm::Attachment {
        // There's a lot more useful stuff to expose.
        MimeType(content_type via QString::from):       "type",
        Data(attachment_path via qstring_from_option):  "data",
    }
}

#[derive(QObject, Default)]
pub struct AttachmentListModel {
    base: qt_base_class!(trait QAbstractListModel),
    attachments: Vec<orm::Attachment>,

    count: qt_property!(i32; NOTIFY rowCountChanged READ row_count),

    /// Gets the nth item of the model, serialized as byte array
    get: qt_method!(fn(&self, idx: i32) -> QByteArray),

    open: qt_method!(fn(&self, idx: i32)),

    rowCountChanged: qt_signal!(),
}

impl AttachmentListModel {
    pub fn new(attachments: Vec<orm::Attachment>) -> Self {
        Self {
            attachments,
            ..Default::default()
        }
    }

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

impl QAbstractListModel for AttachmentListModel {
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
