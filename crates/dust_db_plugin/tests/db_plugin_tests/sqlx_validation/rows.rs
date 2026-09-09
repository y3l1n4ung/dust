//! Validation of row types and the libraries their mappings live in.

use super::*;

/// Where a row type's mapping lives, and when it is missing.
#[path = "rows/mapping.rs"]
mod mapping;

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
        vec![query_execute("SELECT no_such_column FROM users", 0, 10)],
    );

    let diagnostics =
        validate_in_package(&register_plugin(), &[&database, &rows, &queries], &queries);

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("SQLx rejected `queryExecute`")),
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
        vec![query_execute("SELECT * FROM missing_table", 0, 10)],
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
