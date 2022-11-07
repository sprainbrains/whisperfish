#![allow(non_snake_case)]

use std::collections::HashMap;

use libsignal_service::push_service::DeviceInfo;

use qmetaobject::prelude::*;

use super::*;

#[derive(QObject, Default)]
pub struct DeviceModel {
    base: qt_base_class!(trait QAbstractListModel),
    content: Vec<DeviceInfo>,
}

define_model_roles! {
    enum DeviceRoles for DeviceInfo {
        Id(id):                                     "id",
        Name(name via qstring_from_option)  :       "name",
        Created(created via qdatetime_from_chrono): "created",
        LastSeen(last_seen via qdate_from_chrono):  "lastSeen",
    }
}

impl DeviceModel {
    pub fn set_devices(&mut self, content: Vec<DeviceInfo>) {
        (self as &mut dyn QAbstractListModel).begin_reset_model();
        self.content = content;
        (self as &mut dyn QAbstractListModel).end_reset_model();
    }
}

impl QAbstractListModel for DeviceModel {
    fn row_count(&self) -> i32 {
        self.content.len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let role = DeviceRoles::from(role);
        role.get(&self.content[index.row() as usize])
    }

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        DeviceRoles::role_names()
    }
}
