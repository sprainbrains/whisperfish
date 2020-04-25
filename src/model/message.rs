use qmetaobject::*;

#[derive(QObject, Default)]
pub struct MessageModel {
    base: qt_base_class!(trait QObject),
}
