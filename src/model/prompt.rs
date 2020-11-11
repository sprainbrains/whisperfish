use std::future::Future;

use qmetaobject::*;

// XXX code duplication.

#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct Prompt {
    base: qt_base_class!(trait QObject),
    promptPhoneNumber: qt_signal!(),
    promptVerificationCode: qt_signal!(),
    promptPassword: qt_signal!(),

    phoneNumber: qt_method!(fn(&self, phoneNumber: QString)),
    verificationCode: qt_method!(fn(&self, code: QString)),
    password: qt_method!(fn(&self, password: QString)),
    resetPeerIdentity: qt_method!(fn(&self, confirm: QString)),

    password_listeners: Vec<futures::channel::oneshot::Sender<QString>>,
    code_listeners: Vec<futures::channel::oneshot::Sender<QString>>,
    phone_number_listeners: Vec<futures::channel::oneshot::Sender<QString>>,
}

impl Prompt {
    #[allow(non_snake_case)]
    fn phoneNumber(&mut self, phone_number: QString) {
        for listener in self.phone_number_listeners.drain(..) {
            if let Err(_) = listener.send(phone_number.clone()) {
                log::warn!("Request for password fulfilled, but nobody listens.");
            }
        }
    }

    #[allow(non_snake_case)]
    fn verificationCode(&mut self, code: QString) {
        for listener in self.code_listeners.drain(..) {
            if let Err(_) = listener.send(code.clone()) {
                log::warn!("Request for password fulfilled, but nobody listens.");
            }
        }
    }

    #[allow(non_snake_case)]
    fn password(&mut self, password: QString) {
        for listener in self.password_listeners.drain(..) {
            if listener.send(password.clone()).is_err() {
                log::warn!("Request for password fulfilled, but nobody listens.");
            }
        }
    }

    #[allow(non_snake_case)]
    fn resetPeerIdentity(&self, _confirm: QString) {}

    pub fn ask_password(&mut self) -> impl Future<Output = Option<QString>> {
        self.promptPassword();

        let (sender, receiver) = futures::channel::oneshot::channel();

        self.password_listeners.push(sender);

        async {
            match receiver.await {
                Ok(pwd) => Some(pwd),
                Err(_e) => {
                    log::error!("Password prompt was canceled");
                    None
                }
            }
        }
    }

    pub fn ask_phone_number(&mut self) -> impl Future<Output = Option<QString>> {
        self.promptPhoneNumber();

        let (sender, receiver) = futures::channel::oneshot::channel();

        self.phone_number_listeners.push(sender);

        async {
            match receiver.await {
                Ok(pwd) => Some(pwd),
                Err(_e) => {
                    log::error!("Phone number prompt was canceled");
                    None
                }
            }
        }
    }

    pub fn ask_verification_code(&mut self) -> impl Future<Output = Option<QString>> {
        self.promptVerificationCode();

        let (sender, receiver) = futures::channel::oneshot::channel();

        self.code_listeners.push(sender);

        async {
            match receiver.await {
                Ok(pwd) => Some(pwd),
                Err(_e) => {
                    log::error!("Code prompt was canceled");
                    None
                }
            }
        }
    }
}
