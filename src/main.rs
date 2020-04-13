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

    app.set_property("AppVersion".into(), version.into());
    // app.set_object_property("Prompt", prompt);
    // app.set_object_property("SettingsBridge", settings);
    // app.set_object_property("FilePicker", filePicker);
    // app.set_object_property("SessionModel", sessionModel);
    // app.set_object_property("MessageModel", messageModel);
    // app.set_object_property("ContactModel", contactModel);
    // app.set_object_property("DeviceModel", deviceModel);
    // app.set_object_property("SetupWorker", setupWorker);
    // app.set_object_property("ClientWorker", clientWorker);
    // app.set_object_property("SendWorker", sendWorker);

    app.set_source(SailfishApp::path_to("qml/harbour-whisperfish.qml".into()));
    app.show();
    app.exec();
}
