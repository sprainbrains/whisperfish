use qmetaobject::*;
use std::rc::Rc;

#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct SetupWorker {
    base: qt_base_class!(trait QObject),
    registrationSuccess: qt_signal!(),
    invalidDatastore: qt_signal!(),
    invalidPhoneNumber: qt_signal!(),
    clientFailed: qt_signal!(),
}

impl SetupWorker {
    pub async fn run(this: Rc<QObjectBox<Self>>) {
        log::info!("SetupWorker::run");

        //let _store = store::Storage::open(&store::default_location()?)?;
        this.pinned().borrow().clientFailed();
    }
}
