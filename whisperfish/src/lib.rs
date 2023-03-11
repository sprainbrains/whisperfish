#![recursion_limit = "512"]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use crate::diesel::connection::SimpleConnection;
pub mod actor;
pub mod config;
pub mod gui;
pub mod model;
pub mod platform;
pub mod qtlog;
pub mod schema;
pub mod store;
pub mod worker;

pub fn user_agent() -> String {
    format!("Whisperfish/{}", env!("CARGO_PKG_VERSION"))
}

pub fn millis_to_naive_chrono(ts: u64) -> chrono::NaiveDateTime {
    chrono::NaiveDateTime::from_timestamp_opt((ts / 1000) as i64, ((ts % 1000) * 1_000_000) as u32)
        .unwrap()
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

/// Checks if the db contains foreign key violations.
pub fn check_foreign_keys(db: &mut diesel::SqliteConnection) -> Result<(), anyhow::Error> {
    use diesel::prelude::*;
    use diesel::sql_types::*;

    #[derive(Queryable, QueryableByName, Debug)]
    #[allow(dead_code)]
    pub struct ForeignKeyViolation {
        #[diesel(sql_type = Text)]
        table: String,
        #[diesel(sql_type = Integer)]
        rowid: i32,
        #[diesel(sql_type = Text)]
        parent: String,
        #[diesel(sql_type = Integer)]
        fkid: i32,
    }

    db.batch_execute("PRAGMA foreign_keys = ON;").unwrap();
    let violations: Vec<ForeignKeyViolation> = diesel::sql_query("PRAGMA main.foreign_key_check;")
        .load(db)
        .unwrap();

    if !violations.is_empty() {
        anyhow::bail!(
            "There are foreign key violations. Here the are: {:?}",
            violations
        );
    } else {
        Ok(())
    }
}
