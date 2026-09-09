//! Column aliases, nullability overrides and the offline cache.

use super::*;

/// A `foo!` alias names the column `foo`, not a column called `foo!`.
///
/// The marker overrides what the database inferred about nullability, which a
/// `LEFT JOIN` otherwise gets wrong. It is part of the alias, so it reaches
/// describe as part of the name and has to come off before the column is
/// matched against a row class.
#[test]
fn accepts_a_column_alias_carrying_a_nullability_override() {
    let root = temp_root("sqlx_alias_override");
    write_sqlite_project(
        &root,
        r#"
Future<UserProfile> find(Pool db, int id) {
  return queryAs<UserProfile>(
    r'SELECT id, display_name FROM users WHERE id = $1',
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
            r#"SELECT id as "id!", display_name as "display_name!" FROM users WHERE id = $1"#,
            1,
            "fetchOne",
            10,
        )],
    );
    let diagnostics = validate_alone(&register_plugin(), &library);

    assert_eq!(diagnostics, Vec::new());

    let _ = fs::remove_dir_all(root);
}

/// A `?` marker makes a column nullable, and a non-nullable field is told.
///
/// The finding is a warning: the database is often right about nullability and
/// the marker exists for when it is not, so this is worth reading rather than
/// worth failing a build over.
#[test]
fn warns_when_a_nullable_column_reads_into_a_non_nullable_field() {
    let root = temp_root("sqlx_nullable_column");
    write_sqlite_project(
        &root,
        r#"
Future<UserProfile> find(Pool db, int id) {
  return queryAs<UserProfile>(
    r'SELECT id, display_name FROM users WHERE id = $1',
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
            r#"SELECT id, display_name as "display_name?" FROM users WHERE id = $1"#,
            1,
            "fetchOne",
            10,
        )],
    );
    let diagnostics = validate_alone(&register_plugin(), &library);

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("reads nullable column `display_name` into non-nullable")),
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
