#![recursion_limit = "512"]

#[macro_use]
extern crate cpp;

#[macro_use]
extern crate diesel;

use actix::prelude::*;

use qmetaobject::*;

mod qrc;

mod sfos;

mod model;
mod worker;

mod schema;

mod settings;
use settings::Settings;

mod gui;
mod store;

fn main() -> Result<(), failure::Error> {
    let mut sys = System::new("whisperfish");
    env_logger::init();
    qrc::load();

    sfos::TokioQEventDispatcher::install();
    let app = gui::WhisperfishApp::new();

    Arbiter::spawn(worker::SetupWorker::run(app.setup_worker.clone()));
    sys.block_on(async {
        app.run().await.unwrap();
    });

    log::info!("Shut down.");

    Ok(())
}
