use std::cell::RefCell;
#[cfg(feature = "sailfish")]
use std::rc::Rc;

#[cfg(not(feature = "harbour"))]
use std::{
    fs::{create_dir_all, remove_file},
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use crate::store::Storage;
#[allow(unused_imports)] // XXX: review
use crate::{actor, config::Settings, model, worker};

#[cfg(feature = "sailfish")]
use sailors::sailfishapp::QmlApp;

use qmeta_async::with_executor;
use qmetaobject::prelude::*;

use actix::prelude::*;

#[derive(actix::Message, Clone)]
#[rtype(result = "()")]
pub struct StorageReady {
    pub storage: crate::store::Storage,
}

#[derive(QObject)]
#[allow(non_snake_case)]
pub struct AppState {
    base: qt_base_class!(trait QObject),

    closed: bool,
    setActive: qt_method!(fn(&self)),
    isClosed: qt_method!(fn(&self) -> bool),
    activate: qt_signal!(),

    pub may_exit: bool,
    #[cfg(not(feature = "harbour"))]
    setMayExit: qt_method!(fn(&self, value: bool)),

    isAutostartEnabled: qt_method!(fn(&self) -> bool),
    #[cfg(not(feature = "harbour"))]
    setAutostartEnabled: qt_method!(fn(&self, value: bool)),

    isHarbour: qt_method!(fn(&self) -> bool),
}

impl AppState {
    #[allow(non_snake_case)]
    #[with_executor]
    fn setActive(&mut self) {
        self.closed = false;
    }

    #[allow(non_snake_case)]
    #[with_executor]
    pub fn isClosed(&self) -> bool {
        self.closed
    }

    #[with_executor]
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    #[with_executor]
    pub fn set_closed(&mut self) {
        self.closed = true;
    }

    #[with_executor]
    pub fn activate_hidden_window(&mut self, may_exit: bool) {
        if self.closed {
            self.activate();
            // if may_exit = true, we may already be dead when QML sets this, so we set it now.
            self.closed = false;
            self.may_exit = may_exit;
        }
    }

    #[cfg(not(feature = "harbour"))]
    #[allow(non_snake_case)]
    #[with_executor]
    fn setMayExit(&mut self, value: bool) {
        self.may_exit = value;
    }

    #[cfg(not(feature = "harbour"))]
    #[with_executor]
    fn systemd_dir() -> PathBuf {
        let sdir = dirs::config_dir()
            .expect("config directory")
            .join("systemd/user/post-user-session.target.wants/");
        if !sdir.exists() {
            create_dir_all(&sdir).unwrap();
        }
        sdir
    }

    #[cfg(not(feature = "harbour"))]
    #[allow(non_snake_case)]
    #[with_executor]
    fn setAutostartEnabled(&self, enabled: bool) {
        if enabled {
            let _ = symlink(
                Path::new("/usr/lib/systemd/user/harbour-whisperfish.service"),
                Self::systemd_dir().join("harbour-whisperfish.service"),
            );
        } else {
            let _ = remove_file(Self::systemd_dir().join("harbour-whisperfish.service"));
        }
    }

    #[cfg(not(feature = "harbour"))]
    #[allow(non_snake_case)]
    #[with_executor]
    fn isAutostartEnabled(&self) -> bool {
        Self::systemd_dir()
            .join("harbour-whisperfish.service")
            .exists()
    }

    #[cfg(feature = "harbour")]
    #[allow(non_snake_case)]
    #[with_executor]
    fn isAutostartEnabled(&self) -> bool {
        false
    }

    #[allow(non_snake_case)]
    #[with_executor]
    fn isHarbour(&mut self) -> bool {
        cfg!(feature = "harbour")
    }

    #[cfg(feature = "sailfish")]
    #[with_executor]
    fn new() -> Self {
        Self {
            base: Default::default(),
            closed: false,
            may_exit: true,
            setActive: Default::default(),
            isClosed: Default::default(),
            isHarbour: Default::default(),
            activate: Default::default(),
            #[cfg(not(feature = "harbour"))]
            setMayExit: Default::default(),
            isAutostartEnabled: Default::default(),
            #[cfg(not(feature = "harbour"))]
            setAutostartEnabled: Default::default(),
        }
    }
}

pub struct WhisperfishApp {
    pub app_state: QObjectBox<AppState>,
    pub session_actor: Addr<actor::SessionActor>,
    pub message_actor: Addr<actor::MessageActor>,
    pub contact_model: QObjectBox<model::ContactModel>,
    pub prompt: QObjectBox<model::Prompt>,
    pub file_picker: QObjectBox<model::FilePicker>,

    pub client_actor: Addr<worker::ClientActor>,
    pub setup_worker: QObjectBox<worker::SetupWorker>,

    pub settings: QObjectBox<Settings>,

    pub storage: RefCell<Option<Storage>>,
}

impl WhisperfishApp {
    pub async fn storage_ready(&self) {
        let storage = self.storage.borrow().as_ref().unwrap().clone();
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

#[cfg(feature = "sailfish")]
fn long_version() -> String {
    let pkg = env!("CARGO_PKG_VERSION");
    let rpm_version = env!("RPM_VERSION");

    // if not CI, append [commit]-dirty
    // CI changes the version name, because of RPM, so we can just use that.
    if let (Some(_ref_name), Some(_job_id)) =
        (option_env!("CI_COMMIT_REF_NAME"), option_env!("CI_JOB_ID"))
    {
        format!("v{}", pkg)
    } else {
        format!("v{}", rpm_version)
    }
}

#[cfg(feature = "sailfish")]
pub fn run(config: crate::config::SignalConfig) -> Result<(), anyhow::Error> {
    qmeta_async::run(|| {
        let (app, _whisperfish) = with_executor(|| -> anyhow::Result<_> {
            // XXX this arc thing should be removed in the future and refactored
            let config = std::sync::Arc::new(config);

            let mut app = QmlApp::application("harbour-whisperfish".into());
            let long_version: QString = long_version().into();
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
                file_picker: QObjectBox::new(model::FilePicker::default()),

                setup_worker: QObjectBox::new(worker::SetupWorker::default()),

                settings: QObjectBox::new(Settings::default()),

                storage: RefCell::new(None),
            });

            app.set_property("AppVersion".into(), version.into());
            app.set_property("LongAppVersion".into(), long_version.into());
            let ci_job_url: Option<QString> = option_env!("CI_JOB_URL").map(Into::into);
            let ci_job_url = ci_job_url.map(Into::into).unwrap_or(false.into());
            app.set_property("CiJobUrl".into(), ci_job_url);

            app.set_object_property("Prompt".into(), whisperfish.prompt.pinned());
            app.set_object_property("SettingsBridge".into(), whisperfish.settings.pinned());
            app.set_object_property("FilePicker".into(), whisperfish.file_picker.pinned());
            app.set_object_property("ContactModel".into(), whisperfish.contact_model.pinned());
            app.set_object_property("SetupWorker".into(), whisperfish.setup_worker.pinned());
            app.set_object_property("AppState".into(), whisperfish.app_state.pinned());

            app.set_source(QmlApp::path_to("qml/harbour-whisperfish.qml".into()));

            if config.autostart
                && !whisperfish
                    .settings
                    .pinned()
                    .borrow()
                    .get_bool("quit_on_ui_close")
                && cfg!(not(feature = "harbour"))
            {
                // keep the ui closed until needed on auto-start
                whisperfish.app_state.pinned().borrow_mut().may_exit = false;
                whisperfish.app_state.pinned().borrow_mut().set_closed();
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
