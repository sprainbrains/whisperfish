use std::rc::Rc;

use failure::*;
use qmetaobject::*;

use crate::gui::WhisperfishApp;
use crate::settings::SignalConfig;
use crate::store::{self, Storage};

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
    registered: qt_property!(bool; NOTIFY setupChanged),
    locked: qt_property!(bool; NOTIFY setupChanged),
    encryptedKeystore: qt_property!(bool; NOTIFY setupChanged),
    localId: qt_property!(QString; NOTIFY setupChanged),
    identity: qt_property!(QString; NOTIFY setupChanged),

    /// Emitted when any of the properties change.
    setupChanged: qt_signal!(),

    pub config: Option<SignalConfig>,
}

impl SetupWorker {
    pub async fn run(app: Rc<WhisperfishApp>) {
        log::info!("SetupWorker::run");
        let this = app.setup_worker.pinned();

        let identity_path = crate::store::default_location()
            .unwrap()
            .join("storage")
            .join("identity")
            .join("identity_key");

        // Check registration
        if identity_path.is_file() {
            log::info!("identity_key found, assuming registered");
            this.borrow_mut().registered = true;
        } else {
            log::info!("identity_key not found");
        }

        this.borrow_mut().config = match SetupWorker::read_config(app.clone()).await {
            Ok(config) => Some(config),
            Err(e) => {
                log::error!("Error reading config: {:?}", e);
                this.borrow().clientFailed();
                return;
            }
        };

        let config = this.borrow().config.as_ref().unwrap().clone();

        log::debug!("config: {:?}", config);
        // XXX: nice formatting?
        this.borrow_mut().phoneNumber = config.tel.into();

        // Open storage
        if let Err(e) = SetupWorker::setup_storage(app.clone()).await {
            log::error!("Error setting up storage: {}", e);
            this.borrow().clientFailed();
            return;
        }
        app.storage_ready().await;

        this.borrow().setupChanged();
    }

    async fn read_config(_app: Rc<WhisperfishApp>) -> Result<SignalConfig, Error> {
        // XXX non-existing file?
        let conf_dir = dirs::config_dir().ok_or(format_err!("Could not find config directory."))?;
        let signal_config_file = conf_dir.join("harbour-whisperfish").join("config.yml");

        if let Ok(file) = std::fs::File::open(&signal_config_file) {
            Ok(serde_yaml::from_reader(file)?)
        } else {
            let file = std::fs::File::create(signal_config_file)?;
            let contents = SignalConfig {
                tel: String::new(),
                // XXX
                server: String::new(),
                root_ca: String::new(),
                proxy_server: String::new(),
                verification_type: "voice".into(),
                storage_dir: "".into(),
                unencrypted_storage: false,
                storage_password: "".into(),
                log_level: "debug".into(),
                user_agent: "Whisperfish".into(),
                always_trust_peer_id: false,
            };
            serde_yaml::to_writer(file, &contents)?;
            Ok(contents)
        }
    }

    async fn setup_storage(app: Rc<WhisperfishApp>) -> Result<(), Error> {
        let settings = app.settings.pinned();

        let storage = if settings.borrow().get_bool("encrypt_database") {
            let password: String = app
                .prompt
                .pinned()
                .borrow_mut()
                .ask_password()
                .await
                .ok_or(format_err!("No password provided"))?
                .into();

            Storage::open_with_password(&store::default_location()?, password).await?
        } else {
            Storage::open(&store::default_location()?)?
        };

        *app.storage.borrow_mut() = Some(storage);

        Ok(())
    }
}
