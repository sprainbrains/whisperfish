use qmetaobject::*;

#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct SendWorker {
    base: qt_base_class!(trait QObject),
    messageSent: qt_signal!(),
    promptResetPeerIdentity: qt_signal!(),
}
