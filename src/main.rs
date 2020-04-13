#![recursion_limit="256"]

#[macro_use]
extern crate cpp;

use qmetaobject::*;

mod qrc;

mod sfos;

use sfos::*;

fn main() {
    env_logger::init();

    qrc::load();

    let mut app = SailfishApp::application("harbour-whisperfish".into());
    log::info!("SailfishApp::application loaded");
    let version: QString = "0.6.0".into(); // XXX source from Cargo.toml
    app.set_title("Whisperfish".into());
    app.set_application_version(version.clone());
    app.install_default_translator().unwrap();

    let mut engine = app.engine();
    engine.set_property("AppVersion".into(), version.into());
    // engine.set_object_property("Prompt", prompt);
    // engine.set_object_property("SettingsBridge", settings);
    // engine.set_object_property("FilePicker", filePicker);
    // engine.set_object_property("SessionModel", sessionModel);
    // engine.set_object_property("MessageModel", messageModel);
    // engine.set_object_property("ContactModel", contactModel);
    // engine.set_object_property("DeviceModel", deviceModel);
    // engine.set_object_property("SetupWorker", setupWorker);
    // engine.set_object_property("ClientWorker", clientWorker);
    // engine.set_object_property("SendWorker", sendWorker);

    app.set_source(SailfishApp::path_to("qrc:/qml/harbour-whisperfish.qml".into()));
    app.exec();
}
