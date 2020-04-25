use qmetaobject::*;

#[derive(QObject, Default)]
pub struct DeviceModel {
    base: qt_base_class!(trait QObject),
}
