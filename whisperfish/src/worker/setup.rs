use crate::gui::WhisperfishApp;
use crate::store::{Storage, TrustLevel};
use anyhow::Context;
use libsignal_service::push_service::{DeviceId, DEFAULT_DEVICE_ID};
use phonenumber::PhoneNumber;
use qmetaobject::prelude::*;
use std::rc::Rc;

pub struct RegistrationResult {
    regid: u32,
    phonenumber: PhoneNumber,
    uuid: String,
    device_id: DeviceId,
    identity_key_pair: Option<libsignal_protocol::IdentityKeyPair>,
    profile_key: Option<Vec<u8>>,
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
    deviceId: qt_property!(u32; NOTIFY setupChanged),

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
        app.settings_bridge.pinned().borrow_mut().defaults();

        // XXX: nice formatting?
        this.borrow_mut().phoneNumber = config.get_tel_clone().into();
        this.borrow_mut().uuid = config.get_uuid_clone().into();
        this.borrow_mut().deviceId = config.get_device_id().into();

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
            config.set_device_id(this.borrow().deviceId);
            // write changed config to file here
            // XXX handle return value here appropriately !!!
            config.write_to_file().expect("cannot write to config file");
        } else if let Err(e) = SetupWorker::setup_storage(app.clone(), &config).await {
            log::error!("Error setting up storage: {}", e);
            this.borrow().clientFailed();
            return;
        }

        app.storage_ready().await;
        app.app_state.pinned().borrow_mut().setMayExit(
            app.settings_bridge
                .pinned()
                .borrow()
                .get_bool("quit_on_ui_close"),
        );

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
            SetupWorker::register_as_primary(app.clone(), config, &password, &signaling_key).await?
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
        this.uuid = reg.uuid.clone().into();
        this.deviceId = reg.device_id.device_id;

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

        if let Some(profile_key) = reg.profile_key {
            storage.update_profile_key(
                Some(&e164),
                Some(&reg.uuid),
                &profile_key,
                TrustLevel::Certain,
            );
        }

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
            device_id: DeviceId {
                device_id: DEFAULT_DEVICE_ID,
            },
            identity_key_pair: None,
            profile_key: None,
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
                }
                res = res_fut => {
                    let res = res??;
                    return Ok(RegistrationResult {
                        regid: res.registration_id,
                        phonenumber: res.phone_number,
                        uuid: res.uuid,
                        device_id: res.device_id,
                        identity_key_pair: Some(res.identity_key_pair),
                        profile_key: Some(res.profile_key),
                    });
                }
                complete => return Err(anyhow::Error::msg("Linking to device completed without any result")),
            }
        }
    }
}
