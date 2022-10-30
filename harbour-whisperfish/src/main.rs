use whisperfish::*;

use anyhow::Context;
use dbus::blocking::Connection;
use std::time::Duration;

use single_instance::SingleInstance;
use structopt::StructOpt;
use harbour_whisperfish::model::prompt::PromptBox;

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
    /// `RUST_LOG=libsignal_service=trace,libsignal_service_actix=trace,whisperfish=trace`.
    #[structopt(short, long)]
    verbose: bool,

    /// Whether whisperfish was launched from autostart
    #[structopt(short, long)]
    prestart: bool,

    // sailjail only accepts -prestart on the command line as optional argument, structopt however
    // only supports --prestart.
    // See: https://github.com/clap-rs/clap/issues/1210
    // and https://github.com/sailfishos/sailjail/commit/8a239de9451685a82a2ee17fef0c1d33a089c28c
    // XXX: Get rid of this when the situation changes
    #[structopt(short = "r", hidden = true, parse(from_occurrences))]
    _r: u32,
    #[structopt(short = "e", hidden = true)]
    _e: bool,
    #[structopt(short = "s", hidden = true)]
    _s: bool,
    #[structopt(short = "a", hidden = true)]
    _a: bool,
    #[structopt(short = "t", hidden = true, parse(from_occurrences))]
    _t: u32,
}

fn main() {
    // Migrate the config file from
    // ~/.config/harbour-whisperfish/config.yml to
    // ~/.config/be.rubdos/harbour-whisperfish/config.yml
    match config::SignalConfig::migrate_config() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("Could not migrate config file: {}", e);
        }
    };

    // Migrate the QSettings file from
    // ~/.config/harbour-whisperfish/harbour-whisperfish.conf to
    // ~/.config/be.rubdos/harbour-whisperfish/harbour-whisperfish.conf
    match config::SignalConfig::migrate_qsettings() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("Could not migrate QSettings file: {}", e);
        }
    };

    // Read config file or get a default config
    let mut config = match config::SignalConfig::read_from_file() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Config file not found: {}", e);
            config::SignalConfig::default()
        }
    };

    // Migrate the db and storage folders from
    // ~/.local/share/harbour-whisperfish/[...] to
    // ~/.local/share/rubdos.be/harbour-whisperfish/[...]
    match config::SignalConfig::migrate_storage() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("Could not migrate db and storage: {}", e);
            std::process::exit(1);
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
    if opt.prestart {
        config.autostart = true;
    }
    config.override_captcha = opt.captcha;

    // Initiate logger facility
    if config.verbose {
        env_logger::Builder::from_default_env()
            .filter_module("libsignal_service_actix", log::LevelFilter::Trace)
            .filter_module("libsignal_service", log::LevelFilter::Trace)
            .filter_module("whisperfish", log::LevelFilter::Trace)
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

    for dir in &[
        settings.get_string("attachment_dir"),
        settings.get_string("camera_dir"),
    ] {
        let path = std::path::Path::new(dir.trim());
        if !path.exists() {
            std::fs::create_dir_all(path)
                .with_context(|| format!("Could not create dir: {}", path.display()))?;
        }
    }

    // This will panic here if feature `sailfish` is not enabled
    let prompt = Box::new(PromptBox::new());
    gui::run(config, prompt).unwrap();

    log::info!("Shut down.");

    Ok(())
}
