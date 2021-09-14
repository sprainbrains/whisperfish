use crate::gui::WhisperfishApp;
use crate::store::Storage;
use anyhow::Context;
use qmetaobject::prelude::*;
use std::rc::Rc;

#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct SetupWorker {
    base: qt_base_class!(trait QObject),

    registrationSuccess: qt_signal!(),
    invalidDatastore: qt_signal!(),
    invalidPhoneNumber: qt_signal!(),
    clientFailed: qt_signal!(),
    setupComplete: qt_signal!(),

    phoneNumber: qt_property!(QString; NOTIFY setupChanged),
    uuid: qt_property!(QString; NOTIFY setupChanged),

    registered: qt_property!(bool; NOTIFY setupChanged),
    locked: qt_property!(bool; NOTIFY setupChanged),
    encryptedKeystore: qt_property!(bool; NOTIFY setupChanged),
    localId: qt_property!(QString; NOTIFY setupChanged),
    identity: qt_property!(QString; NOTIFY setupChanged),

    useVoice: qt_property!(bool; NOTIFY setupChanged),

    /// Emitted when any of the properties change.
    setupChanged: qt_signal!(),
}

impl SetupWorker {
    const MAX_PASSWORD_ENTER_ATTEMPTS: i8 = 3;

    pub async fn run(app: Rc<WhisperfishApp>, config: std::sync::Arc<crate::config::SignalConfig>) {
        log::info!("SetupWorker::run");
        let this = app.setup_worker.pinned();

        // Check registration
        if config.get_identity_dir().is_file() {
            log::info!("identity_key found, assuming registered");
            this.borrow_mut().registered = true;
        } else {
            log::info!("identity_key not found");
            this.borrow_mut().registered = false;
        }
        this.borrow().setupChanged();

        // Defaults does not override unset settings
        app.settings.pinned().borrow_mut().defaults();

        // XXX: nice formatting?
        this.borrow_mut().phoneNumber = config.get_tel_clone().into();
        this.borrow_mut().uuid = config.get_uuid_clone().into();

        if !this.borrow().registered {
            if let Err(e) = SetupWorker::register(app.clone(), &config).await {
                log::error!("Error in registration: {}", e);
                this.borrow().clientFailed();
                return;
            }
            this.borrow_mut().registered = true;
            this.borrow().setupChanged();
            // change fields in config struct
            config.set_tel(this.borrow().phoneNumber.to_string());
            config.set_uuid(this.borrow().uuid.to_string());
            // write changed config to file here
            // XXX handle return value here appropriately !!!
            config.write_to_file().expect("cannot write to config file");
        } else if let Err(e) = SetupWorker::setup_storage(app.clone(), &config).await {
            log::error!("Error setting up storage: {}", e);
            this.borrow().clientFailed();
            return;
        }

        app.storage_ready().await;

        #[cfg(not(feature = "harbour"))]
        {
            app.app_state.pinned().borrow_mut().may_exit =
                app.settings.pinned().borrow().get_bool("quit_on_ui_close");
        }

        this.borrow().setupChanged();
        this.borrow().setupComplete();
    }

    async fn open_storage(
        app: Rc<WhisperfishApp>,
        config: &crate::config::SignalConfig,
    ) -> Result<Storage, anyhow::Error> {
        let res = Storage::open(&config.get_share_dir().to_owned().into(), None).await;
        if res.is_ok() {
            return res;
        }

        app.app_state
            .pinned()
            .borrow_mut()
            .activate_hidden_window(true);

        for i in 1..=SetupWorker::MAX_PASSWORD_ENTER_ATTEMPTS {
            let password: String = app
                .prompt
                .pinned()
                .borrow_mut()
                .ask_password()
                .await
                .context("No password provided")?
                .into();

            match Storage::open(&config.get_share_dir().to_owned().into(), Some(password)).await {
                Ok(storage) => return Ok(storage),
                Err(error) => log::error!(
                    "Attempt {} of opening encrypted storage failed: {}",
                    i,
                    error
                ),
            }
        }

        log::error!("Error setting up storage: too many bad password attempts");
        res
    }

    async fn setup_storage(
        app: Rc<WhisperfishApp>,
        config: &crate::config::SignalConfig,
    ) -> Result<(), anyhow::Error> {
        let storage = SetupWorker::open_storage(app.clone(), config).await?;

        *app.storage.borrow_mut() = Some(storage);

        Ok(())
    }

    async fn register(
        app: Rc<WhisperfishApp>,
        config: &crate::config::SignalConfig,
    ) -> Result<(), anyhow::Error> {
        let this = app.setup_worker.pinned();

        app.app_state
            .pinned()
            .borrow_mut()
            .activate_hidden_window(true);

        let storage_password: String = app
            .prompt
            .pinned()
            .borrow_mut()
            .ask_password()
            .await
            .context("No password code provided")?
            .into();

        let number = loop {
            let number: String = app
                .prompt
                .pinned()
                .borrow_mut()
                .ask_phone_number()
                .await
                .context("No phone number provided")?
                .into();

            match phonenumber::parse(None, number) {
                Ok(number) => break number,
                Err(e) => {
                    log::warn!("Could not parse phone number: {}", e);
                    this.borrow().invalidPhoneNumber();
                }
            }
        };

        let e164 = number.format().mode(phonenumber::Mode::E164).to_string();
        log::info!("Using phone number: {}", number);
        this.borrow_mut().phoneNumber = e164.clone().into();

        // generate a random 24 bytes password
        use rand::distributions::Alphanumeric;
        use rand::{Rng, RngCore};
        let rng = rand::thread_rng();
        let password: String = rng.sample_iter(&Alphanumeric).take(24).collect();
        // XXX in rand 0.8, this needs to be a Vec<u8> and be converted afterwards.
        // let password = std::str::from_utf8(&password)?.to_string();

        let mut res = app
            .client_actor
            .send(super::client::Register {
                phonenumber: number.clone(),
                password: password.clone(),
                use_voice: this.borrow().useVoice,
                captcha: None,
            })
            .await??;

        while res == super::client::VerificationCodeResponse::CaptchaRequired {
            let captcha: String = app
                .prompt
                .pinned()
                .borrow_mut()
                .ask_captcha()
                .await
                .context("No captcha result provided")?
                .into();
            res = app
                .client_actor
                .send(super::client::Register {
                    phonenumber: number.clone(),
                    password: password.clone(),
                    use_voice: this.borrow().useVoice,
                    captcha: Some(captcha),
                })
                .await??;
        }

        let code: String = app
            .prompt
            .pinned()
            .borrow_mut()
            .ask_verification_code()
            .await
            .context("No verification code provided")?
            .into();
        let code = code.parse()?;

        let mut rng = rand::thread_rng();
        let mut signaling_key = [0u8; 52];
        rng.fill_bytes(&mut signaling_key);
        let signaling_key = signaling_key;

        let (regid, res) = app
            .client_actor
            .send(super::client::ConfirmRegistration {
                phonenumber: number.clone(),
                password: password.clone(),
                confirm_code: code,
                signaling_key,
            })
            .await??;

        log::info!("Registration result: {:?}", res);

        let mut this = this.borrow_mut();

        let storage_password = if storage_password.is_empty() {
            None
        } else {
            Some(storage_password)
        };

        this.uuid = res.uuid.to_string().into();

        // Install storage
        let storage = Storage::new(
            &config.get_share_dir().to_owned().into(),
            storage_password.as_deref(),
            regid,
            &password,
            signaling_key,
        )
        .await?;
        *app.storage.borrow_mut() = Some(storage);

        Ok(())
    }
}
