#![deny(rust_2018_idioms)]

use actix::prelude::*;
use harbour_whisperfish::*;

fn main() -> Result<(), failure::Error> {
    let mut verbose = false;
    for arg in std::env::args() {
        if arg == "-v" || arg == "--verbose" {
            verbose = true;
        }
    }
    if !verbose {
        env_logger::init()
    } else {
        use log::LevelFilter::Trace;
        env_logger::Builder::from_default_env()
            .filter_module("libsignal_service_actix", Trace)
            .filter_module("libsignal_service", Trace)
            .filter_module("harbour_whisperfish", Trace)
            .init()
    }

    let sys = System::new();

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
