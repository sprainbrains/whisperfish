#![recursion_limit = "512"]

pub mod actor;
pub mod config;
pub mod gui;
pub mod model;
pub mod platform;
pub mod qtlog;
pub mod worker;

pub use whisperfish_core::{check_foreign_keys, millis_to_naive_chrono, schema, store};

pub fn user_agent() -> String {
    format!("Whisperfish/{}", env!("CARGO_PKG_VERSION"))
}

pub fn conf_dir() -> std::path::PathBuf {
    let conf_dir = dirs::config_dir()
        .expect("config directory")
        .join("be.rubdos")
        .join("harbour-whisperfish");

    if !conf_dir.exists() {
        std::fs::create_dir(&conf_dir).unwrap();
    }

    conf_dir
}
