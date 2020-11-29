#![allow(non_snake_case)]

use std::collections::HashMap;

use qmetaobject::*;

use super::*;

#[derive(QObject, Default)]
pub struct DeviceModel {
    base: qt_base_class!(trait QObject),

    reload: qt_method!(fn(&self)),

    content: Vec<Device>,
}

#[derive(Debug, Clone, Default)]
struct Device {
    pub id: i32,
    pub name: String,
    pub created: i64,
    pub last_seen: i64,
}

define_model_roles! {
    enum DeviceRoles for Device {
        Id(id):                                   "id",
        Name(name via QString::from)  :           "name",
        Created(created via qdatetime_from_i64):  "created",
        LastSeen(created via qdatetime_from_i64): "last_seen",
    }
}

impl DeviceModel {
    fn reload(&self) {
        log::info!("DeviceModel::reload() called");
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
