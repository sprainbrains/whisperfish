use std::rc::Rc;
use std::cell::RefCell;

use crate::{model, sfos::SailfishApp, worker, Settings};
use crate::store::Storage;

use actix::prelude::*;
use qmetaobject::*;

pub struct WhisperfishApp {
    pub session_model: QObjectBox<model::SessionModel>,
    pub message_model: QObjectBox<model::MessageModel>,
    pub contact_model: QObjectBox<model::ContactModel>,
    pub device_model: QObjectBox<model::DeviceModel>,
    pub prompt: QObjectBox<model::Prompt>,
    pub file_picker: QObjectBox<model::FilePicker>,

    pub client_worker: QObjectBox<worker::ClientWorker>,
    pub send_worker: QObjectBox<worker::SendWorker>,
    pub setup_worker: QObjectBox<worker::SetupWorker>,

    pub settings: QObjectBox<Settings>,

    pub storage: RefCell<Option<Storage>>,
}

pub async fn run() -> Result<(), failure::Error> {
    let mut app = SailfishApp::application("harbour-whisperfish".into());
    log::info!("SailfishApp::application loaded");
    let version: QString = "0.6.0".into(); // XXX source from Cargo.toml
    app.set_title("Whisperfish".into());
    app.set_application_version(version.clone());
    app.install_default_translator().unwrap();

    let whisperfish = Rc::new(WhisperfishApp {
        session_model: QObjectBox::new(model::SessionModel::default()),
        message_model: QObjectBox::new(model::MessageModel::default()),
        contact_model: QObjectBox::new(model::ContactModel::default()),
        device_model: QObjectBox::new(model::DeviceModel::default()),
        prompt: QObjectBox::new(model::Prompt::default()),
        file_picker: QObjectBox::new(model::FilePicker::default()),

        client_worker: QObjectBox::new(worker::ClientWorker::default()),
        send_worker: QObjectBox::new(worker::SendWorker::default()),
        setup_worker: QObjectBox::new(worker::SetupWorker::default()),

        settings: QObjectBox::new(Settings::default()),

        storage: RefCell::new(None),
    });

    Arbiter::spawn(worker::SetupWorker::run(whisperfish.clone()));

    app.set_property("AppVersion".into(), version.into());

    app.set_object_property("Prompt".into(), whisperfish.prompt.pinned());
    app.set_object_property("SettingsBridge".into(), whisperfish.settings.pinned());
    app.set_object_property("FilePicker".into(), whisperfish.file_picker.pinned());
    app.set_object_property("SessionModel".into(), whisperfish.session_model.pinned());
    app.set_object_property("MessageModel".into(), whisperfish.message_model.pinned());
    app.set_object_property("ContactModel".into(), whisperfish.contact_model.pinned());
    app.set_object_property("DeviceModel".into(), whisperfish.device_model.pinned());
    app.set_object_property("SetupWorker".into(), whisperfish.setup_worker.pinned());
    app.set_object_property("ClientWorker".into(), whisperfish.client_worker.pinned());
    app.set_object_property("SendWorker".into(), whisperfish.send_worker.pinned());

    app.set_source(SailfishApp::path_to("qml/harbour-whisperfish.qml".into()));

    app.show();
    app.exec_async().await;

    Ok(())
}
