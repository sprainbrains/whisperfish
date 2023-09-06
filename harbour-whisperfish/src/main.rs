use anyhow::Context;
use dbus::blocking::Connection;
use signal_hook::{consts::SIGINT, iterator::Signals};
use single_instance::SingleInstance;
use std::{os::unix::prelude::OsStrExt, thread, time::Duration};
use structopt::StructOpt;
use whisperfish::*;

use simplelog::*;

/// Unofficial but advanced Signal client for Sailfish OS
#[derive(StructOpt, Debug)]
#[structopt(name = "harbour.whisperfish")]
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

    /// Whether whisperfish was launched from autostart. Also accepts '-prestart'
    #[structopt(short, long)]
    prestart: bool,

    /// Send a signal to shutdown Whisperfish
    #[structopt(long)]
    quit: bool,
}

fn main() {
    // Ctrl-C --> graceful shutdown
    if let Ok(mut signals) = Signals::new([SIGINT].iter()) {
        thread::spawn(move || {
            let mut terminate = false;
            for _ in signals.forever() {
                if !terminate {
                    log::info!("[SIGINT] Trying to exit gracefully...");
                    terminate = true;
                    dbus_quit_app().ok();
                } else {
                    log::info!("[SIGINT] Exiting forcefully...");
                    std::process::exit(1);
                }
            }
        });
    }

    // Sailjail only accepts -prestart on the command line as optional argument,
    // structopt however only supports --prestart.
    // See: https://github.com/clap-rs/clap/issues/1210
    // and https://github.com/sailfishos/sailjail/commit/8a239de9451685a82a2ee17fef0c1d33a089c28c
    // XXX: Get rid of this when the situation changes
    let args = std::env::args_os().map(|arg| {
        if arg == std::ffi::OsStr::from_bytes(b"-prestart") {
            "--prestart".into()
        } else {
            arg
        }
    });

    // Then, handle command line arguments and overwrite settings from config file if necessary
    let opt = Opts::from_iter(args);

    if opt.quit {
        if let Err(e) = dbus_quit_app() {
            eprintln!("{}", e);
        }
        return;
    }

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
    match config::SettingsBridge::migrate_qsettings() {
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
    match store::Storage::migrate_storage() {
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

    if opt.verbose {
        config.verbose = true;
    }
    if opt.prestart {
        config.autostart = true;
    }
    config.override_captcha = opt.captcha;

    let shared_dir = config.get_share_dir();

    let log_file_path = shared_dir.join(format!(
        "harbour.whisperfish.{}.log",
        // Keep in sync with regex above
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    ));
    let log_file = log_file_path.to_str().expect("log file path");

    // Build simplelog configuration
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
        .set_level_color(Level::Trace, None) // To make other colors pop up better
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

    qtlog::enable();

    const MAX_LOGFILE_COUNT: usize = 5;
    const LOGFILE_REGEX: &str = r"harbour.whisperfish.\d{8}_\d{6}\.log";
    if config.logfile {
        store::Storage::clear_old_logs(&shared_dir, MAX_LOGFILE_COUNT, LOGFILE_REGEX);
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
        "be.rubdos.harbour.whisperfish",
        "/be/rubdos/harbour/whisperfish/app",
        Duration::from_millis(20000),
    );

    proxy.method_call("be.rubdos.harbour.whisperfish.app", "show", ())
}

fn dbus_quit_app() -> Result<(), dbus::Error> {
    log::info!("Calling app.quit() on DBus.");

    let c = Connection::new_session()?;
    let proxy = c.with_proxy(
        "be.rubdos.harbour.whisperfish",
        "/be/rubdos/harbour/whisperfish/app",
        Duration::from_millis(1000),
    );

    proxy.method_call("be.rubdos.harbour.whisperfish.app", "quit", ())
}

fn run_main_app(config: config::SignalConfig) -> Result<(), anyhow::Error> {
    log::info!("Start main app (with autostart = {})", config.autostart);

    // Initialise storage here
    // Right now, we only create the attachment (and storage) directory if necessary
    // With more refactoring there should be probably more initialization here
    // Not creating the storage/attachment directory is fatal and we return here.
    let mut settings = crate::config::SettingsBridge::default();
    settings.migrate_qsettings_paths();

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

    // Push verbose and logfile settings to QSettings...
    settings.set_bool("verbose", config.verbose);
    settings.set_bool("logfile", config.logfile);

    // This will panic here if feature `sailfish` is not enabled
    gui::run(config).unwrap();

    // ...and pull them back after execution.
    match config::SignalConfig::read_from_file() {
        Ok(mut config) => {
            config.verbose = settings.get_verbose();
            config.logfile = settings.get_logfile();
            if let Err(e) = config.write_to_file() {
                log::error!("Could not save config.yml: {}", e)
            };
        }
        Err(e) => log::error!("Could not open config.yml: {}", e),
    };

    log::info!("Shut down.");

    Ok(())
}
