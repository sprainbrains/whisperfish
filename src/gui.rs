use crate::{model, worker, Settings, sfos::SailfishApp, store};
use qmetaobject::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct WhisperfishApp {
    pub session_model: Arc<QObjectBox<model::SessionModel>>,
    pub message_model: Arc<QObjectBox<model::MessageModel>>,
    pub contact_model: Arc<QObjectBox<model::ContactModel>>,
    pub device_model: Arc<QObjectBox<model::DeviceModel>>,
    pub prompt: Arc<QObjectBox<model::Prompt>>,
    pub file_picker: Arc<QObjectBox<model::FilePicker>>,

    pub client_worker: Arc<QObjectBox<worker::ClientWorker>>,
    pub send_worker: Arc<QObjectBox<worker::SendWorker>>,
    pub setup_worker: Arc<QObjectBox<worker::SetupWorker>>,

    pub settings: Arc<QObjectBox<Settings>>,
}

impl WhisperfishApp {
    pub fn new() -> Self {
        let session_model = Arc::new(QObjectBox::new(model::SessionModel::default()));
        let message_model = Arc::new(QObjectBox::new(model::MessageModel::default()));
        let contact_model = Arc::new(QObjectBox::new(model::ContactModel::default()));
        let device_model = Arc::new(QObjectBox::new(model::DeviceModel::default()));
        let prompt = Arc::new(QObjectBox::new(model::Prompt::default()));
        let file_picker = Arc::new(QObjectBox::new(model::FilePicker::default()));

        let client_worker = Arc::new(QObjectBox::new(worker::ClientWorker::default()));
        let send_worker = Arc::new(QObjectBox::new(worker::SendWorker::default()));
        let setup_worker = Arc::new(QObjectBox::new(worker::SetupWorker::default()));

        let settings = Arc::new(QObjectBox::new(Settings::default()));

        Self {
            session_model,
            message_model,
            contact_model,
            device_model,
            prompt,
            file_picker,
            client_worker,
            send_worker,
            setup_worker,

            settings,
        }
    }

    pub async fn run(self) -> Result<(), failure::Error> {
        let Self {
            session_model,
            message_model,
            contact_model,
            device_model,
            prompt,
            file_picker,
            client_worker,
            send_worker,
            setup_worker,

            settings,
        } = self;

        let mut app = SailfishApp::application("harbour-whisperfish".into());
        log::info!("SailfishApp::application loaded");
        let version: QString = "0.6.0".into(); // XXX source from Cargo.toml
        app.set_title("Whisperfish".into());
        app.set_application_version(version.clone());
        app.install_default_translator().unwrap();

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
}
