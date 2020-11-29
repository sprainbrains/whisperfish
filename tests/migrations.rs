#[macro_use]
extern crate diesel_migrations;

use diesel_migrations::Migration;

use diesel::prelude::*;
use rstest::*;

type MigrationList = Vec<(String, Box<dyn Migration + 'static>)>;

#[fixture]
fn empty_db() -> SqliteConnection {
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
            subdir.file_name().unwrap().to_str().unwrap().to_string(),
            diesel_migrations::migration_from(subdir).unwrap(),
        ));
    }

    migrations.sort_by_key(|f| f.0.clone());

    assert!(!migrations.is_empty());

    migrations
}

#[fixture]
fn original_go_db(empty_db: SqliteConnection) -> SqliteConnection {
    let message = r#"create table if not exists message 
            (id integer primary key, session_id integer, source text, message string, timestamp integer,
    sent integer default 0, received integer default 0, flags integer default 0, attachment text, 
            mime_type string, has_attachment integer default 0, outgoing integer default 0)"#;
    let sentq = r#"create table if not exists sentq
		(message_id integer primary key, timestamp timestamp)"#;
    let session = r#"create table if not exists session 
		(id integer primary key, source text, message string, timestamp integer,
		 sent integer default 0, received integer default 0, unread integer default 0,
         is_group integer default 0, group_members text, group_id text, group_name text,
		 has_attachment integer default 0)"#;

    diesel::sql_query(message).execute(&empty_db).unwrap();
    diesel::sql_query(sentq).execute(&empty_db).unwrap();
    diesel::sql_query(session).execute(&empty_db).unwrap();

    empty_db
}

#[fixture]
fn fixed_go_db(empty_db: SqliteConnection, mut migrations: MigrationList) -> SqliteConnection {
    drop(migrations.split_off(3));
    assert_eq!(migrations.len(), 3);
    assert_eq!(migrations[0].0, "2020-04-26-145028_0-5-message");
    assert_eq!(migrations[1].0, "2020-04-26-145033_0-5-sentq");
    assert_eq!(migrations[2].0, "2020-04-26-145036_0-5-session");

    diesel_migrations::run_migrations(
        &empty_db,
        migrations.into_iter().map(|m| m.1),
        &mut std::io::stdout(),
    )
    .unwrap();
    empty_db
}

embed_migrations!();

#[rstest(
    db,
    case::empty_db(empty_db()),
    case::original_go_db(original_go_db(empty_db())),
    case::fixed_go_db(fixed_go_db(empty_db(), migrations()))
)]
fn run_plain_migrations(db: SqliteConnection) {
    embedded_migrations::run(&db).unwrap();
}

#[rstest(
    db,
    case::empty_db(empty_db()),
    case::original_go_db(original_go_db(empty_db())),
    case::fixed_go_db(fixed_go_db(empty_db(), migrations()))
)]
fn one_by_one(db: SqliteConnection, migrations: MigrationList) {
    for (_name, migration) in migrations {
        diesel_migrations::run_migrations(&db, vec![migration], &mut std::io::stdout()).unwrap();
    }

    assert!(!diesel_migrations::any_pending_migrations(&db).unwrap());
}
