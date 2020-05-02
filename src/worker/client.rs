use qmetaobject::*;

#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct ClientWorker {
    base: qt_base_class!(trait QObject),
    messageReceived: qt_signal!(),
    messageReceipt: qt_signal!(),
    notifyMessage: qt_signal!(),
    promptResetPeerIdentity: qt_signal!(),
}
