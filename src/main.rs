#![recursion_limit="256"]

#[macro_use]
extern crate cpp;

use qmetaobject::*;

mod qrc;

mod sfos;

mod model;
mod worker;

mod settings;
use settings::Settings;

mod store;

use sfos::*;

fn main() -> Result<(), failure::Error> {
    env_logger::init();

    qrc::load();

    let mut app = SailfishApp::application("harbour-whisperfish".into());
    log::info!("SailfishApp::application loaded");
    let version: QString = "0.6.0".into(); // XXX source from Cargo.toml
    app.set_title("Whisperfish".into());
    app.set_application_version(version.clone());
    app.install_default_translator().unwrap();

    app.set_property("AppVersion".into(), version.into());

    let session_model = QObjectBox::new(model::SessionModel::default());
    let message_model = QObjectBox::new(model::MessageModel::default());
    let contact_model = QObjectBox::new(model::ContactModel::default());
    let device_model = QObjectBox::new(model::DeviceModel::default());
    let prompt = QObjectBox::new(model::Prompt::default());
    let file_picker = QObjectBox::new(model::FilePicker::default());

    let client_worker = QObjectBox::new(worker::ClientWorker::default());
    let send_worker = QObjectBox::new(worker::SendWorker::default());
    let setup_worker = QObjectBox::new(worker::SetupWorker::default());

    let settings = QObjectBox::new(Settings::default());

    let _store = store::Storage::open(&store::default_location()?, "")?;

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
    app.exec();

    Ok(())
}
