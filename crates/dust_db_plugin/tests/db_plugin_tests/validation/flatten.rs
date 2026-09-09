//! Flattened row targets, and the columns they can duplicate.

use super::*;

#[test]
fn validates_flatten_target_must_be_from_row() {
    let mut class = row_class();
    class.fields = vec![field(
        "address",
        TypeIr::named("Address"),
        vec![config("dust_dart::Sqlx", "(flatten: true)")],
    )];
    class.constructors[0].params = vec![named_param("address", TypeIr::named("Address"), false)];
    let diagnostics = register_plugin().validate(&library(vec![class]));

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("must reference an @FromRow class")),
        "{diagnostics:?}"
    );
}

#[test]
fn accepts_a_flatten_target_declared_in_another_library() {
    let root = temp_root("flatten_across_libraries");
    // The flattened row lives in its own file, which is the normal layout.
    let fragment = library_at(
        &root,
        "lib/address.dart",
        vec![ClassIr {
            name: "Address".to_owned(),
            fields: vec![field("street", TypeIr::string(), Vec::new())],
            constructors: vec![ConstructorIr {
                params: vec![named_param("street", TypeIr::string(), false)],
                ..row_class().constructors[0].clone()
            }],
            ..row_class()
        }],
        vec![],
    );
    let mut owner = row_class();
    owner.fields = vec![field(
        "address",
        TypeIr::named("Address"),
        vec![config("dust_dart::Sqlx", "(flatten: true)")],
    )];
    owner.constructors[0].params = vec![named_param("address", TypeIr::named("Address"), false)];
    let library = library_at(&root, "lib/user.dart", vec![owner], vec![]);

    let diagnostics = validate_in_package(&register_plugin(), &[&fragment, &library], &library);

    assert!(
        !diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("must reference an @FromRow class")),
        "{diagnostics:?}"
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn reports_a_column_a_flattened_library_duplicates() {
    let root = temp_root("flatten_duplicate_column");
    let fragment = library_at(
        &root,
        "lib/address.dart",
        vec![ClassIr {
            name: "Address".to_owned(),
            fields: vec![field("street", TypeIr::string(), Vec::new())],
            constructors: vec![ConstructorIr {
                params: vec![named_param("street", TypeIr::string(), false)],
                ..row_class().constructors[0].clone()
            }],
            ..row_class()
        }],
        vec![],
    );
    let mut owner = row_class();
    // `street` is declared here and again by the flattened row.
    owner.fields = vec![
        field("street", TypeIr::string(), Vec::new()),
        field(
            "address",
            TypeIr::named("Address"),
            vec![config("dust_dart::Sqlx", "(flatten: true)")],
        ),
    ];
    owner.constructors[0].params = vec![
        named_param("street", TypeIr::string(), false),
        named_param("address", TypeIr::named("Address"), false),
    ];
    let library = library_at(&root, "lib/user.dart", vec![owner], vec![]);

    let diagnostics = validate_in_package(&register_plugin(), &[&fragment, &library], &library);

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("street")),
        "{diagnostics:?}"
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn validates_conflicting_sqlx_field_options() {
    let mut json_flatten = row_class();
    json_flatten.fields = vec![field(
        "address",
        TypeIr::named("Address"),
        vec![config("dust_dart::Sqlx", "(flatten: true, json: true)")],
    )];
    json_flatten.constructors[0].params =
        vec![named_param("address", TypeIr::named("Address"), false)];

    let mut try_from_flatten = row_class();
    try_from_flatten.fields = vec![field(
        "status",
        TypeIr::named("UserStatus"),
        vec![config(
            "dust_dart::Sqlx",
            "(flatten: true, tryFrom: const UserStatusFromInt())",
        )],
    )];
    try_from_flatten.constructors[0].params =
        vec![named_param("status", TypeIr::named("UserStatus"), false)];

    let diagnostics = register_plugin().validate(&library(vec![json_flatten, try_from_flatten]));

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("cannot use both `json` and `flatten`")),
        "{diagnostics:?}"
    );
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("cannot use both `tryFrom` and `flatten`")),
        "{diagnostics:?}"
    );
}

#[test]
fn validates_unsupported_plain_row_field_type() {
    let mut class = row_class();
    class.fields = vec![field("tags", TypeIr::named("List<String>"), Vec::new())];
    class.constructors[0].params = vec![named_param("tags", TypeIr::named("List<String>"), false)];
    let diagnostics = register_plugin().validate(&library(vec![class]));

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("unsupported SQLx row field type")),
        "{diagnostics:?}"
    );
}
