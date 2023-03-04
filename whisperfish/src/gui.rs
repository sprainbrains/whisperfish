use crate::platform::{is_harbour, MayExit, QmlApp};
use crate::store::Storage;
use crate::{actor, config::SettingsBridge, model, worker};
use actix::prelude::*;
use qmeta_async::with_executor;
use qmetaobject::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(actix::Message, Clone)]
#[rtype(result = "()")]
pub struct StorageReady {
    pub storage: crate::store::Storage,
}

#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct AppState {
    base: qt_base_class!(trait QObject),

    closed: bool,
    setActive: qt_method!(fn(&self)),
    setClosed: qt_method!(fn(&self)),
    isClosed: qt_method!(fn(&self) -> bool),
    activate: qt_signal!(),

    may_exit: MayExit,
    setMayExit: qt_method!(fn(&self, value: bool)),
    mayExit: qt_method!(fn(&self) -> bool),

    isHarbour: qt_method!(fn(&self) -> bool),

    pub storage: RefCell<Option<Storage>>,
}

impl AppState {
    #[allow(non_snake_case)]
    #[with_executor]
    fn setActive(&mut self) {
        self.closed = false;
    }

    #[allow(non_snake_case)]
    #[with_executor]
    fn isClosed(&self) -> bool {
        self.closed
    }

    #[allow(non_snake_case)]
    #[with_executor]
    fn setClosed(&mut self) {
        self.closed = true;
    }

    #[with_executor]
    pub fn activate_hidden_window(&mut self, may_exit: bool) {
        if self.closed {
            self.activate();
            self.closed = false;
            self.may_exit.set_may_exit(may_exit);
        }
    }

    #[allow(non_snake_case)]
    #[with_executor]
    pub fn setMayExit(&mut self, value: bool) {
        self.may_exit.set_may_exit(value);
    }

    #[allow(non_snake_case)]
    #[with_executor]
    fn mayExit(&mut self) -> bool {
        self.may_exit.may_exit()
    }

    #[allow(non_snake_case)]
    #[with_executor]
    fn isHarbour(&mut self) -> bool {
        is_harbour()
    }

    #[with_executor]
    fn new() -> Self {
        Self {
            base: Default::default(),
            closed: false,
            may_exit: MayExit::default(),
            setActive: Default::default(),
            isClosed: Default::default(),
            setClosed: Default::default(),
            isHarbour: Default::default(),
            activate: Default::default(),
            setMayExit: Default::default(),
            mayExit: Default::default(),

            storage: RefCell::default(),
        }
    }
}

pub struct WhisperfishApp {
    pub app_state: QObjectBox<AppState>,
    pub session_actor: Addr<actor::SessionActor>,
    pub message_actor: Addr<actor::MessageActor>,
    pub contact_model: QObjectBox<model::ContactModel>,
    pub prompt: QObjectBox<model::Prompt>,

    pub client_actor: Addr<worker::ClientActor>,
    pub setup_worker: QObjectBox<worker::SetupWorker>,

    pub settings_bridge: QObjectBox<SettingsBridge>,
}

impl WhisperfishApp {
    pub async fn storage_ready(&self) {
        let storage = self
            .app_state
            .pinned()
            .borrow()
            .storage
            .borrow()
            .as_ref()
            .unwrap()
            .clone();
        let msg = StorageReady { storage };

        futures::join! {
            async {
                if let Err(e) = self.session_actor
                    .send(msg.clone()).await {
                    log::error!("Error handling StorageReady: {}", e);
                }
            },
            async {
                if let Err(e) = self.message_actor
                    .send(msg.clone()).await {
                    log::error!("Error handling StorageReady: {}", e);
                }
            },
            async {
                if let Err(e) = self.client_actor
                    .send(msg.clone()).await {
                    log::error!("Error handling StorageReady: {}", e);
                }
            }
        };
    }
}

macro_rules! cstr {
    ($s:expr) => {
        &std::ffi::CString::new($s).unwrap() as &std::ffi::CStr
    };
}

pub fn run(config: crate::config::SignalConfig, long_version: String) -> Result<(), anyhow::Error> {
    qmeta_async::run(|| {
        let (app, _whisperfish) = with_executor(|| -> anyhow::Result<_> {
            // XXX this arc thing should be removed in the future and refactored
            let config = std::sync::Arc::new(config);

            // Register types
            {
                let uri = cstr!("be.rubdos.whisperfish");
                qml_register_type::<model::Sessions>(uri, 1, 0, cstr!("Sessions"));
                qml_register_type::<model::Session>(uri, 1, 0, cstr!("Session"));
                qml_register_type::<model::Message>(uri, 1, 0, cstr!("Message"));
                qml_register_type::<model::Recipient>(uri, 1, 0, cstr!("Recipient"));
                qml_register_type::<model::Group>(uri, 1, 0, cstr!("Group"));
                qml_register_type::<model::Attachment>(uri, 1, 0, cstr!("Attachment"));
                qml_register_type::<model::Reactions>(uri, 1, 0, cstr!("Reactions"));
            }

            let mut app = QmlApp::application("harbour-whisperfish".into());
            let long_version: QString = QString::from(long_version);
            log::info!("QmlApp::application loaded - version {}", long_version);
            let version: QString = env!("CARGO_PKG_VERSION").into();
            app.set_title("Whisperfish".into());
            app.set_application_version(version.clone());
            app.install_default_translator().unwrap();

            // XXX Spaghetti
            let session_actor = actor::SessionActor::new(&mut app).start();
            let client_actor = worker::ClientActor::new(
                &mut app,
                session_actor.clone(),
                std::sync::Arc::clone(&config),
            )?
            .start();
            let message_actor = actor::MessageActor::new(&mut app, client_actor.clone()).start();

            let whisperfish = Rc::new(WhisperfishApp {
                app_state: QObjectBox::new(AppState::new()),
                session_actor,
                message_actor,
                client_actor,
                contact_model: QObjectBox::new(model::ContactModel::default()),
                prompt: QObjectBox::new(model::Prompt::default()),

                setup_worker: QObjectBox::new(worker::SetupWorker::default()),

                settings_bridge: QObjectBox::new(SettingsBridge::default()),
            });

            app.set_property("AppVersion".into(), version.into());
            app.set_property("LongAppVersion".into(), long_version.into());
            let ci_job_url: Option<QString> = option_env!("CI_JOB_URL").map(Into::into);
            let ci_job_url = ci_job_url.map(Into::into).unwrap_or_else(|| false.into());
            app.set_property("CiJobUrl".into(), ci_job_url);

            app.set_object_property("Prompt".into(), whisperfish.prompt.pinned());
            app.set_object_property(
                "SettingsBridge".into(),
                whisperfish.settings_bridge.pinned(),
            );
            app.set_object_property("ContactModel".into(), whisperfish.contact_model.pinned());
            app.set_object_property("SetupWorker".into(), whisperfish.setup_worker.pinned());
            app.set_object_property("AppState".into(), whisperfish.app_state.pinned());

            // We need to decied when to close the app based on the current setup state and
            // background service configuration. We do that in QML in the lastWindowClosed signal
            // emitted from the main QtGuiApplication object, since the corresponding app object in
            // rust is occupied running the main loop.
            // XXX: find a way to set quit_on_last_window_closed from SetupWorker and Settings at
            // runtime to get rid of the QML part here.
            app.set_quit_on_last_window_closed(false);
            app.promote_gui_app_to_qml_context("RootApp".into());

            // We need harbour-whisperfish.qml for the QML-only signalcaptcha application
            // so we have to use another filename for the main QML file for Whisperfish.
            app.set_source(QmlApp::path_to("qml/harbour-whisperfish-main.qml".into()));

            if config.autostart
                && !whisperfish
                    .settings_bridge
                    .pinned()
                    .borrow()
                    .get_bool("quit_on_ui_close")
                && !is_harbour()
            {
                // keep the ui closed until needed on auto-start
                whisperfish
                    .app_state
                    .pinned()
                    .borrow_mut()
                    .setMayExit(false);
                whisperfish.app_state.pinned().borrow_mut().setClosed();
            } else {
                app.show_full_screen();
            }

            actix::spawn(worker::SetupWorker::run(
                whisperfish.clone(),
                std::sync::Arc::clone(&config),
            ));

            Ok((app, whisperfish))
        })
        .expect("setup application");

        app.exec()
    })
}
