use std::rc::Rc;

use crate::{model, worker, Settings, sfos::SailfishApp, store};

use qmetaobject::*;
use actix::prelude::*;

pub async fn run() -> Result<(), failure::Error> {
    let mut app = SailfishApp::application("harbour-whisperfish".into());
    log::info!("SailfishApp::application loaded");
    let version: QString = "0.6.0".into(); // XXX source from Cargo.toml
    app.set_title("Whisperfish".into());
    app.set_application_version(version.clone());
    app.install_default_translator().unwrap();

    let session_model = Rc::new(QObjectBox::new(model::SessionModel::default()));
    let message_model = Rc::new(QObjectBox::new(model::MessageModel::default()));
    let contact_model = Rc::new(QObjectBox::new(model::ContactModel::default()));
    let device_model = Rc::new(QObjectBox::new(model::DeviceModel::default()));
    let prompt = Rc::new(QObjectBox::new(model::Prompt::default()));
    let file_picker = Rc::new(QObjectBox::new(model::FilePicker::default()));

    let client_worker = Rc::new(QObjectBox::new(worker::ClientWorker::default()));
    let send_worker = Rc::new(QObjectBox::new(worker::SendWorker::default()));
    let setup_worker = Rc::new(QObjectBox::new(worker::SetupWorker::default()));

    let settings = Rc::new(QObjectBox::new(Settings::default()));

    Arbiter::spawn(worker::SetupWorker::run(setup_worker.clone()));

    app.set_property("AppVersion".into(), version.into());


    app.set_object_property("Prompt".into(), prompt.pinned());
    app.set_object_property("SettingsBridge".into(), settings.pinned());
    app.set_object_property("FilePicker".into(), file_picker.pinned());
    app.set_object_property("SessionModel".into(), session_model.pinned());
    app.set_object_property("MessageModel".into(), message_model.pinned());
    app.set_object_property("ContactModel".into(), contact_model.pinned());
    app.set_object_property("DeviceModel".into(), device_model.pinned());
    app.set_object_property("SetupWorker".into(), setup_worker.pinned());
    app.set_object_property("ClientWorker".into(), client_worker.pinned());
    app.set_object_property("SendWorker".into(), send_worker.pinned());

    app.set_source(SailfishApp::path_to("qml/harbour-whisperfish.qml".into()));

    app.show();
    app.exec_async().await;

    Ok(())
}
