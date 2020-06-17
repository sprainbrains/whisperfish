#![recursion_limit = "512"]

#[macro_use]
extern crate cpp;

#[macro_use]
extern crate diesel;

pub mod qrc;

pub mod sfos;

pub mod actor;
pub mod model;

pub mod worker;

pub mod schema;

pub mod settings;

pub mod gui;
pub mod store;
