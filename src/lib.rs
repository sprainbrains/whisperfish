#![recursion_limit = "512"]

#[macro_use]
extern crate cpp;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

pub mod sfos;

pub mod actor;
pub mod model;

pub mod worker;

pub mod schema;

pub mod settings;

pub mod gui;
pub mod store;

pub fn millis_to_naive_chrono(ts: u64) -> chrono::NaiveDateTime {
    chrono::NaiveDateTime::from_timestamp((ts / 1000) as i64, ((ts % 1000) * 1_000_000) as u32)
}

pub fn conf_dir() -> std::path::PathBuf {
    let conf_dir = dirs::config_dir()
        .expect("config directory")
        .join("harbour-whisperfish");

    if !conf_dir.exists() {
        std::fs::create_dir(&conf_dir).unwrap();
    }

    conf_dir
}
