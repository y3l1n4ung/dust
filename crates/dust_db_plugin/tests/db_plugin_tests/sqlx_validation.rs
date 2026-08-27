use std::fs;

use dust_db_plugin::{register_plugin, register_validating_plugin};
use dust_ir::{ClassIr, TypeIr};
use dust_plugin_api::{MetadataOutput, PluginExecutionMode};

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
            query_raw("SELECT id, display_name FROM users", 0, 10),
            query_execute("UPDATE users SET display_name = $1 WHERE id = $2", 2, 20),
        ],
    );
    let diagnostics = validate_alone(&register_plugin(), &library);

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

    let library = library_with_queries(
        &root,
        vec![database_class()],
        vec![query_raw("SELECT * FROM missing_table", 0, 10)],
    );
    let diagnostics = validate_alone(&register_plugin(), &library);

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

    let library = library_with_queries(
        &root,
        vec![database_class()],
        vec![query_scalar(
            TypeIr::int(),
            "SELECT COUNT(*) FROM users",
            0,
            "fetchOne",
            10,
        )],
    );
    let diagnostics = validate_alone(
        &register_validating_plugin(PluginExecutionMode::offline(MetadataOutput::ReadOnly)),
        &library,
    );

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("offline query metadata cache is missing")),
        "{diagnostics:?}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn describes_queries_in_a_library_without_the_database_class() {
    let root = temp_root("sqlx_split_libraries");
    write_sqlite_project(&root, "");

    // The layout a real project uses: three files, one concern each.
    let database = library_at(&root, "lib/database.dart", vec![database_class()], vec![]);
    let rows = library_at(
        &root,
        "lib/user.dart",
        vec![simple_user_row_class()],
        vec![],
    );
    let queries = library_at(
        &root,
        "lib/users_repo.dart",
        vec![],
        vec![query_raw("SELECT no_such_column FROM users", 0, 10)],
    );

    let diagnostics =
        validate_in_package(&register_plugin(), &[&database, &rows, &queries], &queries);

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("SQLx rejected `queryRaw`")),
        "{diagnostics:?}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn reports_row_columns_missing_from_a_query_in_another_library() {
    let root = temp_root("sqlx_split_row_columns");
    write_sqlite_project(&root, "");

    let database = library_at(&root, "lib/database.dart", vec![database_class()], vec![]);
    let rows = library_at(
        &root,
        "lib/user.dart",
        vec![simple_user_row_class()],
        vec![],
    );
    let queries = library_at(
        &root,
        "lib/users_repo.dart",
        vec![],
        vec![query_as(
            "UserProfile",
            "SELECT id FROM users WHERE id = $1",
            1,
            "fetchOne",
            10,
        )],
    );

    let diagnostics =
        validate_in_package(&register_plugin(), &[&database, &rows, &queries], &queries);

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("does not return required column `display_name`")),
        "{diagnostics:?}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_query_in_a_package_with_no_database_is_left_alone() {
    let root = temp_root("sqlx_no_database");
    write_sqlite_project(&root, "");

    let queries = library_at(
        &root,
        "lib/users_repo.dart",
        vec![],
        vec![query_raw("SELECT * FROM missing_table", 0, 10)],
    );

    assert_eq!(
        validate_in_package(&register_plugin(), &[&queries], &queries),
        Vec::new()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_a_package_declaring_more_than_one_database() {
    let root = temp_root("sqlx_two_databases");
    write_sqlite_project(&root, "");

    let first = library_at(&root, "lib/database.dart", vec![database_class()], vec![]);
    let second = library_at(
        &root,
        "lib/reporting.dart",
        vec![ClassIr {
            name: "ReportingDatabase".to_owned(),
            ..database_class()
        }],
        vec![],
    );

    let plugin = register_plugin();
    let libraries = [&first, &second];
    let reported = [&first, &second]
        .into_iter()
        .flat_map(|library| validate_in_package(&plugin, &libraries, library))
        .filter(|diagnostic| diagnostic.message.contains("more than one SqlxDatabase"))
        .collect::<Vec<_>>();

    // Once for the package, not once per library that happens to declare one.
    assert_eq!(reported.len(), 1, "{reported:?}");
    assert!(
        reported[0]
            .message
            .contains("(`AppDatabase`, `ReportingDatabase`)"),
        "{reported:?}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_a_row_type_with_no_row_mapping() {
    let root = temp_root("sqlx_unmapped_row");
    write_sqlite_project(&root, "");

    let database = library_at(&root, "lib/database.dart", vec![database_class()], vec![]);
    // No row class anywhere in the package declares `Widget`.
    let queries = library_at(
        &root,
        "lib/widgets_repo.dart",
        vec![],
        vec![query_as(
            "Widget",
            "SELECT id FROM users WHERE id = $1",
            1,
            "fetchOne",
            10,
        )],
    );

    let diagnostics = validate_in_package(&register_plugin(), &[&database, &queries], &queries);

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("queryAs<Widget> row type has no row mapping")),
        "{diagnostics:?}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn accepts_a_row_type_whose_mapping_lives_in_another_library() {
    let root = temp_root("sqlx_mapped_row_elsewhere");
    write_sqlite_project(&root, "");

    let database = library_at(&root, "lib/database.dart", vec![database_class()], vec![]);
    let rows = library_at(
        &root,
        "lib/user.dart",
        vec![simple_user_row_class()],
        vec![],
    );
    let queries = library_at(
        &root,
        "lib/users_repo.dart",
        vec![],
        vec![query_as(
            "UserProfile",
            "SELECT id, display_name FROM users WHERE id = $1",
            1,
            "fetchOne",
            10,
        )],
    );

    assert_eq!(
        validate_in_package(&register_plugin(), &[&database, &rows, &queries], &queries),
        Vec::new()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn accepts_an_unmapped_row_type_when_the_call_brings_its_own_mapper() {
    let root = temp_root("sqlx_own_mapper");
    write_sqlite_project(&root, "");

    let database = library_at(&root, "lib/database.dart", vec![database_class()], vec![]);
    let mut call = query_as(
        "Widget",
        "SELECT id FROM users WHERE id = $1",
        1,
        "fetchOne",
        10,
    );
    call.has_row_mapper_argument = true;
    let queries = library_at(&root, "lib/widgets_repo.dart", vec![], vec![call]);

    let diagnostics = validate_in_package(&register_plugin(), &[&database, &queries], &queries);

    assert!(
        !diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("row type has no row mapping")),
        "{diagnostics:?}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn accepts_query_as_row_which_needs_no_mapping() {
    let root = temp_root("sqlx_row_type_row");
    write_sqlite_project(&root, "");

    let database = library_at(&root, "lib/database.dart", vec![database_class()], vec![]);
    let queries = library_at(
        &root,
        "lib/rows_repo.dart",
        vec![],
        vec![query_as("Row", "SELECT id FROM users", 0, "fetchAll", 10)],
    );

    let diagnostics = validate_in_package(&register_plugin(), &[&database, &queries], &queries);

    assert!(
        !diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("row type has no row mapping")),
        "{diagnostics:?}"
    );

    let _ = fs::remove_dir_all(root);
}
