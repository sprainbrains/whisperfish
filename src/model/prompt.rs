use qmetaobject::*;

#[derive(QObject, Default)]
pub struct Prompt {
    base: qt_base_class!(trait QObject),
}
