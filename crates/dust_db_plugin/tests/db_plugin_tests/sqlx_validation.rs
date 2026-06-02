use std::fs;

use dust_db_plugin::{register_plugin, register_plugin_with_options};
use dust_ir::TypeIr;
use dust_plugin_api::DustPlugin;

use super::support::*;

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
    let diagnostics = register_plugin().validate(&library_with_queries(
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
            query_raw("SELECT id, display_name FROM users", 0, 10),
            query_execute("UPDATE users SET display_name = $1 WHERE id = $2", 2, 20),
        ],
    ));

    assert_eq!(diagnostics, Vec::new());
    let cache = fs::read_to_string(root.join(".dart_tool/dust/db_query_cache_v2.json")).unwrap();
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
            ("SELECT id, display_name FROM users", "raw"),
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

    let diagnostics = register_plugin().validate(&library_with_queries(
        &root,
        vec![database_class()],
        vec![query_raw("SELECT * FROM missing_table", 0, 10)],
    ));

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("SQLx rejected `queryRaw`")),
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

    let diagnostics = register_plugin().validate(&library_with_queries(
        &root,
        vec![simple_user_row_class(), database_class()],
        vec![query_as(
            "UserProfile",
            "SELECT id FROM users WHERE id = $1",
            1,
            "fetchOne",
            10,
        )],
    ));

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("does not return required column `display_name`")),
        "{diagnostics:?}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_offline_query_without_metadata_cache() {
    let root = temp_root("sqlx_missing_offline_cache");
    write_sqlite_project(
        &root,
        r#"
Future<int> count(Pool db) {
  return queryScalar<int>(
    r'SELECT COUNT(*) FROM users',
    [],
  ).fetchOne(db);
}
"#,
    );

    let diagnostics = register_plugin_with_options(true, false).validate(&library_with_queries(
        &root,
        vec![database_class()],
        vec![query_scalar(
            TypeIr::int(),
            "SELECT COUNT(*) FROM users",
            0,
            "fetchOne",
            10,
        )],
    ));

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("offline query metadata cache is missing")),
        "{diagnostics:?}"
    );

    let _ = fs::remove_dir_all(root);
}
