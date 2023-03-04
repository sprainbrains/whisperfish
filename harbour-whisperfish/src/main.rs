use anyhow::Context;
use dbus::blocking::Connection;
use simplelog::*;
use single_instance::SingleInstance;
use std::time::Duration;
use whisperfish::*;

const HELP: &str = "USAGE:
  harbour-whisperfish [FLAGS] [OPTIONS]

FLAGS:
  -h, --help
        Prints help information

  -p, --prestart
        Whether whisperfish was launched from autostart

  -V, --version
        Prints version information

  -v, --verbose
        Verbosity.

        Equivalent with setting `QT_LOGGING_TO_CONSOLE=1
        RUST_LOG=libsignal_service=trace,libsignal_service_actix=trace,whisperfish=trace`.

OPTIONS:
  -c, --captcha <captcha>
        Captcha override

        By opening https://signalcaptchas.org/registration/generate.html in a browser, and intercepting the redirect
        (by using the console), it is possible to inject a signalcaptcha URL.

        This is as a work around for https://gitlab.com/whisperfish/whisperfish/-/issues/378";

#[derive(Debug)]
struct Opts {
    captcha: Option<String>,
    verbose: bool,
    prestart: bool,
}

fn parse_args() -> Result<Opts, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    // Help has a higher priority and should be handled separately.
    if pargs.contains(["-h", "--help"]) {
        println!("{}", long_version());
        println!("{}", HELP);
        std::process::exit(0);
    } else if pargs.contains(["-V", "--version"]) {
        println!("{}", long_version());
        std::process::exit(0);
    }

    let args = Opts {
        captcha: pargs.opt_value_from_str(["-c", "--captcha"])?,
        verbose: pargs.contains(["-v", "--verbose"]),
        prestart: pargs.contains(["-prestart", "--prestart"]),
    };

    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Error: unused arguments: {:?}.", remaining);
        std::process::exit(1);
    }

    Ok(args)
}

fn long_version() -> String {
    let pkg = env!("CARGO_PKG_VERSION");

    // If it's tagged, use the tag as-is
    // If it's in CI, use the cargo version with the ref-name and job id appended
    // else, we use whatever git thinks is the version,
    // finally, we fall back on Cargo's version as-is
    if let Some(tag) = option_env!("CI_COMMIT_TAG") {
        // Tags are mainly used for specific versions
        tag.into()
    } else if let (Some(ref_name), Some(job_id)) =
        (option_env!("CI_COMMIT_REF_NAME"), option_env!("CI_JOB_ID"))
    {
        // This is always the fall-back in CI
        format!("v{}-{}-{}", pkg, ref_name, job_id)
    } else if let Some(git_version) = option_env!("GIT_VERSION") {
        // This is normally possible with any build
        git_version.into()
    } else {
        // But if git is not available, we fall back on cargo
        format!("v{}", env!("CARGO_PKG_VERSION"))
    }
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
    let opt = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };
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
    gui::run(config, long_version()).unwrap();

    log::info!("Shut down.");

    Ok(())
}
