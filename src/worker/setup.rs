use qmetaobject::*;

#[derive(QObject, Default)]
pub struct SetupWorker {
    base: qt_base_class!(trait QObject),
}
