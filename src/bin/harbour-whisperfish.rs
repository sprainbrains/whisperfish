#![deny(rust_2018_idioms)]

use actix::prelude::*;
use harbour_whisperfish::*;

use dbus::blocking::Connection;
use std::time::Duration;

fn main() -> Result<(), failure::Error> {
    env_logger::init();

    let mut autostart = false;
    let mut verbose = false;
    let mut ignored = 0;
    for arg in std::env::args() {
        if arg == "autostart" {
            autostart = true;
        } else if arg == "-v" || arg == "--verbose" {
            verbose = true;
        } else {
            ignored += 1;
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

    if ignored > 1 {
        log::warn!("{} console arguments ignored", ignored - 1);
    }

    if let Ok(()) = try_dbus_show_app() {
        return Ok(());
    }

    run_main_app(autostart)
}

fn try_dbus_show_app() -> Result<(), dbus::Error> {
    log::info!("Try calling app.show() on DBus.");

    let c = Connection::new_session()?;
    let proxy = c.with_proxy(
        "be.rubdos.whisperfish.app",
        "/be/rubdos/whisperfish",
        Duration::from_millis(1000),
    );

    proxy.method_call("be.rubdos.whisperfish.app", "show", ())
}

fn run_main_app(is_autostart: bool) -> Result<(), failure::Error> {
    log::info!("Start main app (with autostart = {})", is_autostart);

    let sys = System::new();

    sfos::TokioQEventDispatcher::install();

    sys.block_on(async {
        // Currently not possible, default QmlEngine does not run asynchronous.
        // Soft-blocked on https://github.com/woboq/qmetaobject-rs/issues/102

        #[cfg(feature = "sailfish")]
        gui::run(is_autostart).await.unwrap();
    });

    log::info!("Shut down.");

    Ok(())
}
