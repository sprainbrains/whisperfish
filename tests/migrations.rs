#[macro_use]
extern crate diesel_migrations;

use diesel_migrations::Migration;

use std::path::Path;

use diesel::prelude::*;
use rstest::*;

type MigrationList = Vec<Box<dyn Migration + 'static>>;

#[fixture]
fn db() -> SqliteConnection {
    SqliteConnection::establish(":memory:").unwrap()
}

#[fixture]
fn migrations() -> MigrationList {
    let mut migrations = Vec::new();
    for subdir in std::fs::read_dir("migrations").unwrap() {
        let subdir = subdir.unwrap().path();

        if !subdir.is_dir() {
            log::warn!("Skipping non-migration {:?}", subdir);
            continue;
        }

        migrations.push((
            subdir.to_str().unwrap().to_string(),
            diesel_migrations::migration_from(subdir).unwrap(),
        ));
    }

    migrations.sort_by_key(|f| f.0.clone());

    assert!(!migrations.is_empty());

    migrations.into_iter().map(|f| f.1).collect()
}

embed_migrations!();

#[rstest]
fn run_plain_migrations(db: SqliteConnection) {
    embedded_migrations::run(&db).unwrap();
}

#[rstest]
fn one_by_one(db: SqliteConnection, migrations: MigrationList) {
    for migration in migrations {
        diesel_migrations::run_migrations(&db, vec![migration], &mut std::io::stdout()).unwrap();
    }

    assert!(!diesel_migrations::any_pending_migrations(&db).unwrap());
}
