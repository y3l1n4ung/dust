use dust_ir::SpanIr;
use dust_text::{FileId, TextRange};
use sqlx::sqlite::SqliteConnection;

use super::*;
use crate::plugin::model::{FetchMode, QueryFunction};

/// Describing a query against a live database and checking its columns.
#[path = "tests/describe.rs"]
mod describe;

/// SQLite is a real sqlx backend that needs no server, so the paths this
/// file shares between the two dialects can be exercised in-process. Only
/// connecting to PostgreSQL genuinely needs one.
async fn memory_connection() -> SqliteConnection {
    SqliteConnection::connect("sqlite::memory:")
        .await
        .expect("in-memory SQLite must open")
}

fn migrations_dir(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    for (name, sql) in files {
        fs::write(dir.path().join(name), sql).expect("write migration");
    }
    dir
}

#[test]
fn a_url_for_another_driver_is_ignored_rather_than_opened() {
    // One workspace, two drivers, one environment variable. A SQLite
    // project handed a PostgreSQL URL falls back to its own default.
    let chosen = validation_database_url(
        DbDriver::Sqlite3,
        Some("postgres://user@localhost/app?sslmode=disable".to_owned()),
        Some("sqlite::memory:"),
    );

    assert_eq!(chosen.as_deref(), Ok("sqlite::memory:"));
}

#[test]
fn a_url_for_this_driver_wins_over_the_fallback() {
    let chosen = validation_database_url(
        DbDriver::Postgres,
        Some("postgres://user@localhost/app".to_owned()),
        Some("ignored"),
    );

    assert_eq!(chosen.as_deref(), Ok("postgres://user@localhost/app"));
}

#[test]
fn a_driver_with_no_url_and_no_fallback_says_which_scheme_it_needs() {
    let error = validation_database_url(DbDriver::Postgres, None, None)
        .expect_err("PostgreSQL has no in-memory default");

    assert!(error.contains("`postgres`/`postgresql`"), "{error}");
    assert!(error.contains("--offline"), "{error}");
}

#[tokio::test]
async fn migrations_are_applied_in_name_order() {
    // `0002` depends on `0001` having run, so applying them out of order
    // fails rather than passing quietly.
    let dir = migrations_dir(&[
        ("0001_create.sql", "CREATE TABLE items (id INTEGER);"),
        ("0002_alter.sql", "ALTER TABLE items ADD COLUMN name TEXT;"),
    ]);
    let mut conn = memory_connection().await;

    apply_migrations(&mut conn, dir.path())
        .await
        .expect("migrations must apply");

    conn.execute("SELECT id, name FROM items")
        .await
        .expect("both migrations must have run");
}

#[tokio::test]
async fn a_migration_the_database_refuses_names_the_file() {
    let dir = migrations_dir(&[("0001_broken.sql", "CREATE TABLE (;")]);
    let mut conn = memory_connection().await;

    let error = apply_migrations(&mut conn, dir.path())
        .await
        .expect_err("invalid SQL must fail");

    assert!(error.contains("failed to apply migration"), "{error}");
    assert!(error.contains("0001_broken.sql"), "{error}");
}

/// One query spec, with the fields a describe test cares about.
fn query(sql: &str, function: QueryFunction, parameters: usize) -> QuerySpec {
    QuerySpec {
        function,
        fetch: FetchMode::All,
        sql: sql.to_owned(),
        sql_source_static: true,
        row_type: Some("Item".to_owned()),
        scalar_type: None,
        parameter_count: parameters,
        params_source_is_list: false,
        has_row_mapper_argument: false,
        unsafe_sql_allowed: false,
        span: SpanIr::new(FileId::new(7), TextRange::new(0_u32, 1_u32)),
        display_name: Some("Items.all".to_owned()),
    }
}
