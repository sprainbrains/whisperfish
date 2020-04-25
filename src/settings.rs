use qmetaobject::*;

#[derive(QObject, Default)]
pub struct Settings {
    base: qt_base_class!(trait QObject),
}
