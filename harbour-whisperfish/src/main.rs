use anyhow::Context;
use dbus::blocking::Connection;
use single_instance::SingleInstance;
use std::time::Duration;
use structopt::StructOpt;
use whisperfish::*;

use simplelog::*;

/// Signal attachment downloader for Whisperfish
#[derive(StructOpt, Debug)]
#[structopt(name = "harbour-whisperfish")]
struct Opts {
    /// Captcha override
    ///
    /// By opening https://signalcaptchas.org/registration/generate.html in a browser,
    /// and intercepting the redirect (by using the console),
    /// it is possible to inject a signalcaptcha URL.
    ///
    /// This is as a work around for https://gitlab.com/whisperfish/whisperfish/-/issues/378
    #[structopt(short, long)]
    captcha: Option<String>,

    /// Verbosity.
    ///
    /// Equivalent with setting
    /// `QT_LOGGING_TO_CONSOLE=1 RUST_LOG=libsignal_service=trace,libsignal_service_actix=trace,whisperfish=trace`.
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

    // Build simplelog configuration
    let shared_dir = config.get_share_dir().join("harbour-whisperfish.log");
    let log_file = shared_dir.to_str().expect("log file path");
    let mut log_level = LevelFilter::Warn;
    let mut config_builder = ConfigBuilder::new();

    config_builder
        .set_time_format_str("%Y-%m-%d %H:%M:%S%.3f")
        .add_filter_allow_str("whisperfish")
        .add_filter_allow_str("libsignal_service")
        .add_filter_allow_str("libsignal_service_actix")
        .set_max_level(LevelFilter::Error) // Always show e.g. [INFO]
        .set_thread_level(LevelFilter::Off) // Hide thread info
        .set_location_level(LevelFilter::Off) // Hide filename, row and column
        .set_level_color(Level::Trace, Some(Color::Magenta))
        .set_level_color(Level::Debug, Some(Color::Blue))
        .set_level_color(Level::Info, Some(Color::Green))
        .set_level_color(Level::Warn, Some(Color::Yellow))
        .set_level_color(Level::Error, Some(Color::Red));

    if config.verbose {
        // Enable QML debug output and full backtrace (for Sailjail).
        std::env::set_var("QT_LOGGING_TO_CONSOLE", "1");
        std::env::set_var("RUST_BACKTRACE", "full");
        log_level = LevelFilter::Trace;
    }

    CombinedLogger::init(if config.logfile {
        vec![
            TermLogger::new(
                log_level,
                config_builder.build(),
                TerminalMode::Stderr,
                ColorChoice::Auto,
            ),
            WriteLogger::new(
                log_level,
                config_builder.build(),
                std::fs::File::create(log_file).unwrap(),
            ),
        ]
    } else {
        vec![TermLogger::new(
            log_level,
            config_builder.build(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        )]
    })
    .unwrap();

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
    let settings = crate::config::SettingsBridge::default();

    for dir in &[
        settings.get_string("attachment_dir"),
        settings.get_string("camera_dir"),
        settings.get_string("avatar_dir"),
    ] {
        let path = std::path::Path::new(dir.trim());
        if !path.exists() {
            std::fs::create_dir_all(path)
                .with_context(|| format!("Could not create dir: {}", path.display()))?;
        }
    }

    // This will panic here if feature `sailfish` is not enabled
    gui::run(config).unwrap();

    log::info!("Shut down.");

    Ok(())
}
