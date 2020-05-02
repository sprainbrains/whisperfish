use qmetaobject::*;

#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct Prompt {
    base: qt_base_class!(trait QObject),
    promptPhoneNumber: qt_signal!(),
    promptVerificationCode: qt_signal!(),
    promptPassword: qt_signal!(),
}
