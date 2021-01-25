use std::cell::RefCell;
#[cfg(feature = "sailfish")]
use std::rc::Rc;

use crate::store::Storage;
#[allow(unused_imports)] // XXX: review
use crate::{
    actor, model,
    settings::{Settings, SignalConfig},
    sfos::SailfishApp,
    worker,
};

use actix::prelude::*;
use qmetaobject::*;

#[derive(actix::Message, Clone)]
#[rtype(result = "()")]
pub struct StorageReady(pub Storage, pub SignalConfig);

pub struct WhisperfishApp {
    pub session_actor: Addr<actor::SessionActor>,
    pub message_actor: Addr<actor::MessageActor>,
    pub contact_model: QObjectBox<model::ContactModel>,
    pub device_model: QObjectBox<model::DeviceModel>,
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
        let config = self.setup_worker.pinned().borrow().config.clone().unwrap();
        let msg = StorageReady(storage, config);

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

fn long_version() -> String {
    let pkg = env!("CARGO_PKG_VERSION");
    let commit = env!("VERGEN_SHA_SHORT");

    if let (Some(ref_name), Some(job_id)) =
        (option_env!("CI_COMMIT_REF_NAME"), option_env!("CI_JOB_ID"))
    {
        format!("{}-{}-{}", ref_name, commit, job_id)
    } else {
        format!("v{}-{}-dirty", pkg, commit)
    }
}

#[cfg(feature = "sailfish")]
pub async fn run() -> Result<(), failure::Error> {
    let mut app = SailfishApp::application("harbour-whisperfish".into());
    let long_version: QString = long_version().into();
    log::info!("SailfishApp::application loaded - version {}", long_version);
    let version: QString = env!("CARGO_PKG_VERSION").into();
    app.set_title("Whisperfish".into());
    app.set_application_version(version.clone());
    app.install_default_translator().unwrap();

    // XXX Spaghetti
    let session_actor = actor::SessionActor::new(&mut app).start();
    let client_actor = worker::ClientActor::new(&mut app, session_actor.clone())?.start();
    let message_actor = actor::MessageActor::new(&mut app, client_actor.clone()).start();

    let whisperfish = Rc::new(WhisperfishApp {
        session_actor,
        message_actor,
        client_actor,
        contact_model: QObjectBox::new(model::ContactModel::default()),
        device_model: QObjectBox::new(model::DeviceModel::default()),
        prompt: QObjectBox::new(model::Prompt::default()),
        file_picker: QObjectBox::new(model::FilePicker::default()),

        setup_worker: QObjectBox::new(worker::SetupWorker::default()),

        settings: QObjectBox::new(Settings::default()),

        storage: RefCell::new(None),
    });

    Arbiter::spawn(worker::SetupWorker::run(whisperfish.clone()));

    app.set_property("AppVersion".into(), version.into());
    app.set_property("LongAppVersion".into(), long_version.into());
    let ci_job_url: Option<QString> = option_env!("CI_JOB_URL").map(Into::into);
    let ci_job_url = ci_job_url.map(Into::into).unwrap_or(false.into());
    app.set_property("CiJobUrl".into(), ci_job_url);

    whisperfish.contact_model.pinned().borrow_mut().refresh();

    app.set_object_property("Prompt".into(), whisperfish.prompt.pinned());
    app.set_object_property("SettingsBridge".into(), whisperfish.settings.pinned());
    app.set_object_property("FilePicker".into(), whisperfish.file_picker.pinned());
    app.set_object_property("ContactModel".into(), whisperfish.contact_model.pinned());
    app.set_object_property("DeviceModel".into(), whisperfish.device_model.pinned());
    app.set_object_property("SetupWorker".into(), whisperfish.setup_worker.pinned());

    app.set_source(SailfishApp::path_to("qml/harbour-whisperfish.qml".into()));

    app.show_full_screen();
    app.exec_async().await;

    Ok(())
}
