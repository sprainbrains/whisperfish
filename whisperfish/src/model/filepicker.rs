use qmetaobject::*;

#[derive(QObject, Default)]
pub struct FilePicker {
    base: qt_base_class!(trait QObject),
}
