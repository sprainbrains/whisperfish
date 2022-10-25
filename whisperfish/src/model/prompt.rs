use async_trait::async_trait;
use std::process::Command;

use crate::gui_traits::{PromptApi, Registrable};
use crate::platform::QmlApp;
use qmeta_async::with_executor;
use qmetaobject::prelude::*;

// XXX code duplication.

#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct Prompt {
    base: qt_base_class!(trait QObject),
    promptRegistrationType: qt_signal!(),
    promptPhoneNumber: qt_signal!(),
    promptVerificationCode: qt_signal!(),
    promptPassword: qt_signal!(),
    promptCaptcha: qt_signal!(),
    showLinkQR: qt_signal!(),

    linkingQR: qt_property!(QString; NOTIFY qrChanged),
    qrChanged: qt_signal!(),

    registerAsPrimary: qt_method!(fn(&self, isPrimary: bool)),

    phoneNumber: qt_method!(fn(&self, phoneNumber: QString)),
    verificationCode: qt_method!(fn(&self, code: QString)),
    password: qt_method!(fn(&self, password: QString)),
    captcha: qt_method!(fn(&self, captcha: QString)),
    resetPeerIdentity: qt_method!(fn(&self, confirm: QString)),

    startCaptcha: qt_method!(fn(&self)),

    password_listeners: Vec<futures::channel::oneshot::Sender<QString>>,
    registration_type_listeners: Vec<futures::channel::oneshot::Sender<bool>>,
    code_listeners: Vec<futures::channel::oneshot::Sender<QString>>,
    phone_number_listeners: Vec<futures::channel::oneshot::Sender<QString>>,
    captcha_listeners: Vec<futures::channel::oneshot::Sender<QString>>,
}

impl Prompt {
    #[allow(non_snake_case)]
    #[with_executor]
    fn phoneNumber(&mut self, phone_number: QString) {
        for listener in self.phone_number_listeners.drain(..) {
            if listener.send(phone_number.clone()).is_err() {
                log::warn!("Request for phone number fulfilled, but nobody listens.");
            }
        }
    }

    #[allow(non_snake_case)]
    #[with_executor]
    fn verificationCode(&mut self, code: QString) {
        for listener in self.code_listeners.drain(..) {
            if listener.send(code.clone()).is_err() {
                log::warn!("Request for verification code fulfilled, but nobody listens.");
            }
        }
    }

    #[allow(non_snake_case)]
    #[with_executor]
    fn password(&mut self, password: QString) {
        for listener in self.password_listeners.drain(..) {
            if listener.send(password.clone()).is_err() {
                log::warn!("Request for password fulfilled, but nobody listens.");
            }
        }
    }

    #[allow(non_snake_case)]
    #[with_executor]
    fn captcha(&mut self, captcha: QString) {
        for listener in self.captcha_listeners.drain(..) {
            if listener.send(captcha.clone()).is_err() {
                log::warn!("Request for captcha fulfilled, but nobody listens.");
            }
        }
    }

    #[allow(non_snake_case)]
    #[with_executor]
    fn resetPeerIdentity(&self, _confirm: QString) {}

    #[allow(non_snake_case)]
    #[with_executor]
    fn registerAsPrimary(&mut self, isPrimary: bool) {
        for listener in self.registration_type_listeners.drain(..) {
            if listener.send(isPrimary).is_err() {
                log::warn!("Request for registration type fulfilled, but nobody listens.");
            }
        }
    }

    #[allow(non_snake_case)]
    #[with_executor]
    fn startCaptcha(&mut self) {
        Command::new("/usr/bin/sailfish-qml")
            .args(&["harbour-whisperfish"])
            .spawn()
            .expect("/usr/bin/sailfish-qml not found, libsailfishapp-launcher not installed?");
    }
}

pub struct PromptBox {
    inner: QObjectBox<Prompt>,
}

impl PromptBox {
    pub fn new() -> Self {
        PromptBox {
            inner: QObjectBox::new(Prompt::default()),
        }
    }
}

#[async_trait(?Send)]
impl PromptApi for PromptBox {
    async fn ask_phone_number(&self) -> Option<String> {
        let prompt = self.inner.pinned();
        let mut prompt = prompt.borrow_mut();

        prompt.promptPhoneNumber();

        let (sender, receiver) = futures::channel::oneshot::channel();

        prompt.phone_number_listeners.push(sender);

        match receiver.await {
            Ok(pwd) => Some(pwd.into()),
            Err(_e) => {
                log::error!("Phone number prompt was canceled");
                None
            }
        }
    }

    async fn ask_verification_code(&self) -> Option<String> {
        let prompt = self.inner.pinned();
        let mut prompt = prompt.borrow_mut();

        prompt.promptVerificationCode();

        let (sender, receiver) = futures::channel::oneshot::channel();

        prompt.code_listeners.push(sender);

        match receiver.await {
            Ok(pwd) => Some(pwd.into()),
            Err(_e) => {
                log::error!("Code prompt was canceled");
                None
            }
        }
    }

    async fn ask_captcha(&self) -> Option<String> {
        let prompt = self.inner.pinned();
        let mut prompt = prompt.borrow_mut();

        prompt.promptCaptcha();

        let (sender, receiver) = futures::channel::oneshot::channel();

        prompt.captcha_listeners.push(sender);

        match receiver.await {
            Ok(pwd) => Some(pwd.into()),
            Err(_e) => {
                log::error!("Captcha prompt was canceled");
                None
            }
        }
    }

    async fn ask_registration_type(&self) -> Option<bool> {
        let prompt = self.inner.pinned();
        let mut prompt = prompt.borrow_mut();

        prompt.promptRegistrationType();

        let (sender, receiver) = futures::channel::oneshot::channel();

        prompt.registration_type_listeners.push(sender);

        match receiver.await {
            Ok(pwd) => Some(pwd.into()),
            Err(_e) => {
                log::error!("Registration type prompt was canceled");
                None
            }
        }
    }

    async fn ask_password(&self) -> Option<String> {
        let prompt = self.inner.pinned();
        let mut prompt = prompt.borrow_mut();

        prompt.promptPassword();

        let (sender, receiver) = futures::channel::oneshot::channel();

        prompt.password_listeners.push(sender);

        match receiver.await {
            Ok(pwd) => Some(pwd.into()),
            Err(_e) => {
                log::error!("Password prompt was canceled");
                None
            }
        }
    }

    fn show_link_qr(&self, url: String) {
        let prompt = self.inner.pinned();
        let mut prompt = prompt.borrow_mut();

        let code = qrcode::QrCode::new(url.as_str()).expect("to generate qrcode for linking URI");
        let image_buf = code.render::<image::Luma<u8>>().build();

        // Export generate QR code pixmap data into a PNG data:-URI string
        let mut image_uri = String::from("data:image/png;base64,");
        {
            let mut image_b64enc =
                base64::write::EncoderStringWriter::from(&mut image_uri, base64::STANDARD);
            image::png::PngEncoder::new(&mut image_b64enc)
                .encode(
                    &*image_buf,
                    image_buf.width(),
                    image_buf.height(),
                    <image::Luma<u8> as image::Pixel>::COLOR_TYPE,
                )
                .expect("to write QR code image to data:-URI");
        }

        prompt.linkingQR = QString::from(image_uri);
        prompt.qrChanged();
        prompt.showLinkQR();
    }
}

impl Registrable<QmlApp> for PromptBox {
    fn register_in(&self, target: &mut QmlApp) {
        target.set_object_property("Prompt".into(), self.inner.pinned());
    }
}
