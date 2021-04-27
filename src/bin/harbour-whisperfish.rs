use actix::prelude::*;
use harbour_whisperfish::*;

use anyhow::Context;
use dbus::blocking::Connection;
use std::time::Duration;

fn main() {
    // Read config file or get a default config
    let mut config = match config::SignalConfig::read_from_file() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Config file not found: {}", e);
            config::SignalConfig::default()
        }
    };

    // Write config to initialize a default config
    if let Err(e) = config.write_to_file() {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    // Then, handle command line arguments and overwrite settings from config file if necessary
    config::Args::check_args(&mut config);

    // Initiate logger facility
    if config.verbose {
        env_logger::Builder::from_default_env()
            .filter_module("libsignal_service_actix", log::LevelFilter::Trace)
            .filter_module("libsignal_service", log::LevelFilter::Trace)
            .filter_module("harbour_whisperfish", log::LevelFilter::Trace)
            .init()
    } else {
        env_logger::init()
    }

    match try_dbus_show_app() {
        Ok(_) => return,
        Err(e) => log::error!("{}", e),
    }

    if let Err(e) = run_main_app(config) {
        log::error!("Fatal error: {}", e);
        std::process::exit(1);
    }
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

fn run_main_app(config: config::SignalConfig) -> Result<(), anyhow::Error> {
    log::info!("Start main app (with autostart = {})", config.autostart);

    // Initialise storage here
    // Right now, we only create the attachment (and storage) directory if necessary
    // With more refactoring there should be probably more initialization here
    // Not creating the storage/attachment directory is fatal and we return here.
    let settings = crate::config::Settings::default();
    let dir = settings.get_string("attachment_dir");
    let path = std::path::Path::new(dir.trim());
    if !path.exists() {
        std::fs::create_dir_all(path)
            .with_context(|| format!("Could not create attachment dir: {}", path.display()))?;
    }

    sfos::TokioQEventDispatcher::install();

    let sys = System::new();
    sys.block_on(async {
        // Currently not possible, default QmlEngine does not run asynchronous.
        // Soft-blocked on https://github.com/woboq/qmetaobject-rs/issues/102

        #[cfg(feature = "sailfish")]
        gui::run(config).await.unwrap();
    });

    log::info!("Shut down.");

    Ok(())
}
