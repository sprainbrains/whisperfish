#![recursion_limit = "512"]
#![deny(rust_2018_idioms)]

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

pub fn conf_dir() -> std::path::PathBuf {
    let conf_dir = dirs::config_dir()
        .expect("config directory")
        .join("harbour-whisperfish");

    if !conf_dir.exists() {
        std::fs::create_dir(&conf_dir).unwrap();
    }

    conf_dir
}
