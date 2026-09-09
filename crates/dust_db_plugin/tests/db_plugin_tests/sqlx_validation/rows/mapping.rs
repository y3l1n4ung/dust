//! Where a row type's mapping lives, and when it is missing.

use super::*;

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

#[test]
fn rejects_a_row_class_name_two_libraries_share() {
    let root = temp_root("sqlx_duplicate_row_name");
    write_sqlite_project(&root, "");

    let database = library_at(&root, "lib/database.dart", vec![database_class()], vec![]);
    let rows = library_at(
        &root,
        "lib/user.dart",
        vec![simple_user_row_class()],
        vec![],
    );
    // A second `UserProfile`, in another library of the same package.
    let other = library_at(
        &root,
        "lib/archive.dart",
        vec![ClassIr {
            fields: vec![field("archived", TypeIr::int(), Vec::new())],
            ..simple_user_row_class()
        }],
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

    let plugin = register_plugin();
    let libraries = [&database, &rows, &other, &queries];
    let reported = libraries
        .into_iter()
        .flat_map(|library| validate_in_package(&plugin, &libraries, library))
        .collect::<Vec<_>>();

    let ambiguous = reported
        .iter()
        .filter(|diagnostic| {
            diagnostic
                .message
                .contains("more than one row class named `UserProfile`")
        })
        .collect::<Vec<_>>();
    // Once for the package, naming both files.
    assert_eq!(ambiguous.len(), 1, "{reported:?}");
    assert!(
        ambiguous[0].message.contains("lib/archive.dart")
            && ambiguous[0].message.contains("lib/user.dart"),
        "{ambiguous:?}"
    );
    // And the query is not additionally rejected against whichever won.
    assert!(
        !reported.iter().any(|diagnostic| diagnostic
            .message
            .contains("does not return required column")),
        "{reported:?}"
    );

    let _ = fs::remove_dir_all(root);
}
