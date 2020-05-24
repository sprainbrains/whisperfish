use std::future::Future;

use qmetaobject::*;

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
}

impl Prompt {
    #[allow(non_snake_case)]
    fn phoneNumber(&self, _phone_number: QString) {
    }

    #[allow(non_snake_case)]
    fn verificationCode(&self, _code: QString) {
    }

    #[allow(non_snake_case)]
    fn password(&mut self, password: QString) {
        for listener in self.password_listeners.drain(..) {
            if let Err(_) = listener.send(password.clone()) {
                log::warn!("Request for password fulfilled, but nobody listens.");
            }
        }
    }

    #[allow(non_snake_case)]
    fn resetPeerIdentity(&self, _confirm: QString) {
    }

    pub fn ask_password(&mut self) -> impl Future<Output=Option<QString>> {
        self.promptPassword();

        let (sender, receiver) = futures::channel::oneshot::channel();

        self.password_listeners.push(sender);

        async {
            match receiver.await {
                Ok(pwd) => Some(pwd),
                Err(_e) => {
                    log::error!("Password prompt was canceled");
                    None
                },
            }
        }
    }
}
