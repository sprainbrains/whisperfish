use qmetaobject::*;

#[derive(QObject, Default)]
pub struct ClientWorker {
    base: qt_base_class!(trait QObject),
}
