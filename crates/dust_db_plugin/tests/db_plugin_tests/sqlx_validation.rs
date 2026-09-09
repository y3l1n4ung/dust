use std::fs;

use dust_db_plugin::{register_plugin, register_validating_plugin};
use dust_ir::{ClassIr, TypeIr};
use dust_plugin_api::{MetadataOutput, PluginExecutionMode};

use super::support::*;

/// Column aliases, nullability overrides and the offline cache.
#[path = "sqlx_validation/columns.rs"]
mod columns;

/// Validation of row types and the libraries their mappings live in.
#[path = "sqlx_validation/rows.rs"]
mod rows;

#[test]
fn validates_static_query_calls_against_sqlite_and_writes_cache() {
    let root = temp_root("sqlx_valid_queries");
    write_sqlite_project(
        &root,
        r#"
Future<UserProfile?> find(Pool db, int id) {
  return queryAs<UserProfile>(
    r'SELECT id, display_name FROM users WHERE id = $1',
    [id],
  ).fetchOptional(db);
}

Future<int> count(Pool db) {
  return queryScalar<int>(
    r'SELECT COUNT(*) FROM users',
    [],
  ).fetchOne(db);
}

Future<List<Row>> raw(Pool db) {
  return queryRaw(
    r'SELECT id, display_name FROM users',
    [],
  ).fetch(db);
}

Future<ExecResult> rename(Pool db, String name, int id) {
  return queryExecute(
    r'UPDATE users SET display_name = $1 WHERE id = $2',
    [name, id],
  ).execute(db);
}
"#,
    );
    let library = library_with_queries(
        &root,
        vec![simple_user_row_class(), database_class()],
        vec![
            query_as(
                "UserProfile",
                "SELECT id, display_name FROM users WHERE id = $1",
                1,
                "fetchOptional",
                40,
            ),
            query_scalar(
                TypeIr::int(),
                "SELECT COUNT(*) FROM users",
                0,
                "fetchOne",
                30,
            ),
            query_execute("SELECT id, display_name FROM users", 0, 10),
            query_execute("UPDATE users SET display_name = $1 WHERE id = $2", 2, 20),
        ],
    );
    let diagnostics = validate_alone(&register_plugin(), &library);

    assert_eq!(diagnostics, Vec::new());
    // One cache file per library, so parallel worker threads never write the
    // same path. The name is derived from the library's source path.
    let cache_dir = root.join(".dust_sql");
    let mut cache_files = fs::read_dir(&cache_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    cache_files.sort();
    assert_eq!(cache_files.len(), 1, "{cache_files:?}");
    assert!(
        cache_files[0]
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("db-"),
        "{cache_files:?}"
    );
    let cache = fs::read_to_string(&cache_files[0]).unwrap();
    let cache: serde_json::Value = serde_json::from_str(&cache).unwrap();
    let entries = cache["entries"].as_array().unwrap();
    let query_modes = entries
        .iter()
        .map(|entry| {
            (
                entry["sql"].as_str().unwrap(),
                entry["fetch_mode"].as_str().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        query_modes,
        vec![
            ("SELECT id, display_name FROM users", "execute"),
            ("SELECT COUNT(*) FROM users", "one"),
            (
                "UPDATE users SET display_name = $1 WHERE id = $2",
                "execute"
            ),
            (
                "SELECT id, display_name FROM users WHERE id = $1",
                "optional"
            ),
        ]
    );
    // The driver is part of the key: a cache written here must not satisfy a
    // build targeting another dialect.
    assert!(
        entries
            .iter()
            .all(|entry| entry["driver"].as_str() == Some("sqlite3")),
        "{entries:?}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_sqlx_invalid_static_query() {
    let root = temp_root("sqlx_bad_query");
    write_sqlite_project(
        &root,
        r#"
Future<List<Row>> bad(Pool db) {
  return queryRaw(
    r'SELECT * FROM missing_table',
    [],
  ).fetch(db);
}
"#,
    );

    let library = library_with_queries(
        &root,
        vec![database_class()],
        vec![query_execute("SELECT * FROM missing_table", 0, 10)],
    );
    let diagnostics = validate_alone(&register_plugin(), &library);

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("SQLx rejected `queryExecute`")),
        "{diagnostics:?}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_sqlx_query_missing_required_from_row_column() {
    let root = temp_root("sqlx_missing_column");
    write_sqlite_project(
        &root,
        r#"
Future<UserProfile> find(Pool db, int id) {
  return queryAs<UserProfile>(
    r'SELECT id FROM users WHERE id = $1',
    [id],
  ).fetchOne(db);
}
"#,
    );

    let library = library_with_queries(
        &root,
        vec![simple_user_row_class(), database_class()],
        vec![query_as(
            "UserProfile",
            "SELECT id FROM users WHERE id = $1",
            1,
            "fetchOne",
            10,
        )],
    );
    let diagnostics = validate_alone(&register_plugin(), &library);

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("does not return required column `display_name`")),
        "{diagnostics:?}"
    );

    let _ = fs::remove_dir_all(root);
}
