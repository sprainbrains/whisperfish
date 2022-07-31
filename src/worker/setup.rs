use crate::gui::WhisperfishApp;
use crate::store::Storage;
use anyhow::Context;
use qmetaobject::prelude::*;
use std::rc::Rc;

use phonenumber::PhoneNumber;

pub struct RegistrationResult {
    regid: u32,
    phonenumber: PhoneNumber,
    uuid: String,
    identity_key_pair: Option<libsignal_protocol::IdentityKeyPair>,
}

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
                    "Attempt {} of opening encrypted storage failed: {:?}",
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

        let is_primary: bool = app
            .prompt
            .pinned()
            .borrow_mut()
            .ask_registration_type()
            .await
            .context("No registration type chosen")?;

        // generate a random 24 bytes password
        use rand::distributions::Alphanumeric;
        use rand::{Rng, RngCore};
        let rng = rand::thread_rng();
        let password: String = rng.sample_iter(&Alphanumeric).take(24).collect();
        // XXX in rand 0.8, this needs to be a Vec<u8> and be converted afterwards.
        // let password = std::str::from_utf8(&password)?.to_string();

        // generate a 52 bytes signaling key
        let mut rng = rand::thread_rng();
        let mut signaling_key = [0u8; 52];
        rng.fill_bytes(&mut signaling_key);
        let signaling_key = signaling_key;

        let reg = if is_primary {
            SetupWorker::register_as_primary(app.clone(), config, &password, &signaling_key)
                .await?
        } else {
            SetupWorker::register_as_secondary(app.clone(), &password, &signaling_key).await?
        };

        let mut this = this.borrow_mut();

        let storage_password = if storage_password.is_empty() {
            None
        } else {
            Some(storage_password)
        };

        let e164 = reg
            .phonenumber
            .format()
            .mode(phonenumber::Mode::E164)
            .to_string();
        this.phoneNumber = e164.clone().into();
        this.uuid = reg.uuid.into();

        // Install storage
        let storage = Storage::new(
            &config.get_share_dir().to_owned().into(),
            storage_password.as_deref(),
            reg.regid,
            &password,
            signaling_key,
            reg.identity_key_pair,
        )
        .await?;
        *app.storage.borrow_mut() = Some(storage);

        Ok(())
    }

    async fn register_as_primary(
        app: Rc<WhisperfishApp>,
        config: &crate::config::SignalConfig,
        password: &str,
        signaling_key: &[u8; 52],
    ) -> Result<RegistrationResult, anyhow::Error> {
        let this = app.setup_worker.pinned();

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

        //let e164 = number.format().mode(phonenumber::Mode::E164).to_string();
        //log::info!("Using phone number: {}", number);
        //this.borrow_mut().phoneNumber = e164.clone().into();

        if let Some(captcha) = &config.override_captcha {
            log::info!("Using override captcha {}", captcha);
        }
        let mut res = app
            .client_actor
            .send(super::client::Register {
                phonenumber: number.clone(),
                password: password.to_string(),
                use_voice: this.borrow().useVoice,
                captcha: config.override_captcha.clone(),
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
                    password: password.to_string(),
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

        let (regid, res) = app
            .client_actor
            .send(super::client::ConfirmRegistration {
                phonenumber: number.clone(),
                password: password.to_string(),
                confirm_code: code,
                signaling_key: *signaling_key,
            })
            .await??;

        log::info!("Registration result: {:?}", res);

        Ok(RegistrationResult {
            regid,
            phonenumber: number,
            uuid: res.uuid.to_string(),
            identity_key_pair: Option::None,
        })
    }

    async fn register_as_secondary(
        app: Rc<WhisperfishApp>,
        password: &str,
        signaling_key: &[u8; 52],
    ) -> Result<RegistrationResult, anyhow::Error> {
        use futures::FutureExt;

        let (tx_uri, rx_uri) = futures::channel::oneshot::channel();

        let res_fut = app.client_actor.send(super::client::RegisterLinked {
            device_name: String::from("Whisperfish"),
            password: password.to_string(),
            signaling_key: *signaling_key,
            tx_uri,
        });

        let res_fut = res_fut.fuse();
        let rx_uri = rx_uri.fuse();

        futures::pin_mut!(res_fut, rx_uri);

        loop {
            futures::select! {
                uri_result = rx_uri => {
                    app.prompt
                        .pinned()
                        .borrow_mut()
                        .show_link_qr(uri_result?);
                        //.context("No verification code provided")?
                        //.into();
                }
                res = res_fut => {
                    let res = res??;
                    return Ok(RegistrationResult {
                        regid: res.registration_id,
                        phonenumber: res.phone_number,
                        uuid: res.uuid,
                        identity_key_pair: Some(res.identity_key_pair),
                    });
                }
                complete => return Err(anyhow::Error::msg("Linking to device completed without any result")),
            }
        }

        //let linking_handle = RefCell::new(linking_handle);
        //let succeeded = Cell::new(false);
        //scopeguard::defer! {
        //// Notify UI that we are done
        //linking_handle.borrow_mut().notify_completed(succeeded.get());
        //};

        //let mut completion_fut  = completion_fut.fuse();
        //let mut link_device_fut = link_device_fut.fuse();
        //loop {
        //futures::select! {
        //uri_result = rx => {
        //// Tell UI to display the received provisioning URI
        //linking_handle.borrow_mut().submit_uri(&uri_result?);
        //},

        //link_result = link_device_fut => {
        //let link_result = link_result??;

        //let mut this = this.borrow_mut();
        //let mut cfg = this.config.as_mut().unwrap();
        //cfg.uuid = Some(link_result.uuid.clone());
        //cfg.tel = Some(link_result.phone_number);
        //Self::write_config(app.clone(), cfg).await?;
        //this.uuid = link_result.uuid.into();

        //let context = libsignal_protocol::Context::default();
        //let identity_key_pair = libsignal_protocol::keys::IdentityKeyPair::new(
        //&libsignal_protocol::keys::PublicKey::decode_point(
        //&context, link_result.public_key.as_slice()
        //).unwrap(),
        //&libsignal_protocol::keys::PrivateKey::decode_point(
        //&context, link_result.private_key.as_slice()
        //).unwrap(),
        //).unwrap();

        //// Install storage
        //let storage = Storage::new_with_password(
        //&store::default_location()?,
        //&storage_password,
        //link_result.registration_id,
        //&password,
        //signaling_key,
        //identity_key_pair,
        //).await?;
        //*app.storage.borrow_mut() = Some(storage);

        //// Report success to UI
        //succeeded.set(true);

        //break Ok(());
        //},

        //_ = completion_fut => {
        //// This shouldn't be called if linking succeeded,
        //// since this coroutine will have already exited
        //// after initializing storage
        //break Err(format_err!("Operation aborted by user"));
        //},

        //complete => {
        //// XXX: This is probably unreachable now?
        //break Err(format_err!("Linking to device completed without any result"));
        //}
        //}
        //}
    }
}
