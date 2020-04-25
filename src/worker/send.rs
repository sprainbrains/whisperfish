use qmetaobject::*;

#[derive(QObject, Default)]
pub struct SendWorker {
    base: qt_base_class!(trait QObject),
}
