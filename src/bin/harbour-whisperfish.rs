use harbour_whisperfish::*;

use anyhow::Context;
use dbus::blocking::Connection;
use std::time::Duration;

use single_instance::SingleInstance;

/// Signal attachment downloader for Whisperfish
#[derive(StructOpt, Debug)]
#[structopt(name = "harbour-whisperfish")]
struct Opts {
    /// Captcha override
    ///
    /// By opening https://signalcaptchas.org/registration/generate.html in a browser,
    /// and intercepting the redirect (by using the console), it is possible to inject a reCAPTCHA.
    ///
    /// This is as a work around for https://gitlab.com/whisperfish/whisperfish/-/issues/378
    #[structopt(short, long)]
    captcha: Option<String>,

    /// Verbosity.
    ///
    /// Equivalent with setting
    /// `RUST_LOG=libsignal_service=trace,libsignal_service_actix=trace,harbour_whisperfish=trace`.
    #[structopt(short, long)]
    verbose: bool,

    /// Whether whisperfish was launched from autostart
    #[structopt(long)]
    autostart: bool,
}

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
    let opt = Opts::from_args();
    if opt.verbose {
        config.verbose = true;
    }
    if opt.autostart {
        config.autostart = true;
    }

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

    let instance_lock = SingleInstance::new("whisperfish").unwrap();
    if !instance_lock.is_single() {
        if let Err(e) = dbus_show_app() {
            log::error!("{}", e);
        }
        return;
    }

    if let Err(e) = run_main_app(config) {
        log::error!("Fatal error: {}", e);
        std::process::exit(1);
    }
}

fn dbus_show_app() -> Result<(), dbus::Error> {
    log::info!("Calling app.show() on DBus.");

    let c = Connection::new_session()?;
    let proxy = c.with_proxy(
        "be.rubdos.whisperfish",
        "/be/rubdos/whisperfish/app",
        Duration::from_millis(20000),
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

    // Currently not possible, default QmlEngine does not run asynchronous.
    // Soft-blocked on https://github.com/woboq/qmetaobject-rs/issues/102
    #[cfg(feature = "sailfish")]
    gui::run(config).unwrap();

    log::info!("Shut down.");

    Ok(())
}
