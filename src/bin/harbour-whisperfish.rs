#![deny(rust_2018_idioms)]

use actix::prelude::*;
use harbour_whisperfish::*;

fn main() -> Result<(), failure::Error> {
    let mut sys = System::new("whisperfish");
    env_logger::init();
    qrc::load();

    sfos::TokioQEventDispatcher::install();

    sys.block_on(async {
        // Currently not possible, default QmlEngine does not run asynchronous.
        // Soft-blocked on https://github.com/woboq/qmetaobject-rs/issues/102

        #[cfg(feature = "sailfish")]
        gui::run().await.unwrap();
    });

    log::info!("Shut down.");

    Ok(())
}
