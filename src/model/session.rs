use std::collections::HashMap;

use qmetaobject::*;

#[derive(QObject, Default)]
pub struct SessionModel {
    base: qt_base_class!(trait QAbstractListModel),
}

impl QAbstractListModel for SessionModel {
    fn row_count(&self) -> i32 {
        0
    }

    fn data(&self, _index: QModelIndex, _role: i32) -> QVariant {
        unimplemented!()
    }

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        unimplemented!()
    }
}
