use qmetaobject::*;

#[derive(QObject, Default)]
pub struct ContactModel {
    base: qt_base_class!(trait QObject),
}
