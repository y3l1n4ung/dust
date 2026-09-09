use dust_db_plugin::register_plugin;
use dust_ir::{ClassIr, ConstructorIr, MethodIr, TypeIr};
use dust_plugin_api::DustPlugin;

use super::support::*;

/// Flattened row targets, and the columns they can duplicate.
#[path = "validation/flatten.rs"]
mod flatten;

#[test]
fn rejects_unknown_custom_dao_result_type() {
    let mut dao = dao_class();
    dao.methods[0].return_type = result_type(TypeIr::named("NotARow"));

    let diagnostics = register_plugin().validate(&library(vec![database_class(), dao]));

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("unsupported SqlxDao result type `NotARow`")),
        "{diagnostics:?}"
    );
}

#[test]
fn rejects_invalid_dao_constructor_method_and_parameter_shapes() {
    let mut dao = dao_class();
    dao.constructors.clear();
    dao.methods = vec![
        MethodIr {
            name: "body".to_owned(),
            is_static: false,
            is_external: false,
            return_type: result_type(TypeIr::named("ExecResult")),
            has_body: true,
            body_source: Some("{}".to_owned()),
            params: Vec::new(),
            span: span(),
            traits: Vec::new(),
            configs: vec![config("dust_dart::Query", "(r'DELETE FROM users')")],
        },
        MethodIr {
            name: "namedParam".to_owned(),
            is_static: false,
            is_external: false,
            return_type: result_type(TypeIr::named("ExecResult")),
            has_body: false,
            body_source: None,
            params: vec![named_method_param("id", TypeIr::int(), true)],
            span: span(),
            traits: Vec::new(),
            configs: vec![config(
                "dust_dart::Query",
                "(r'DELETE FROM users WHERE id = $1')",
            )],
        },
        MethodIr {
            name: "dynamicSql".to_owned(),
            is_static: false,
            is_external: false,
            return_type: result_type(TypeIr::named("ExecResult")),
            has_body: false,
            body_source: None,
            params: Vec::new(),
            span: span(),
            traits: Vec::new(),
            configs: vec![config("dust_dart::Query", "(sql)")],
        },
        MethodIr {
            name: "badReturn".to_owned(),
            is_static: false,
            is_external: false,
            return_type: TypeIr::named("Future<void>"),
            has_body: false,
            body_source: None,
            params: Vec::new(),
            span: span(),
            traits: Vec::new(),
            configs: vec![config("dust_dart::Query", "(r'SELECT 1')")],
        },
    ];

    let diagnostics = register_plugin().validate(&library(vec![database_class(), dao]));
    let messages = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();

    assert!(
        messages
            .iter()
            .any(|message| message.contains("must declare `const factory UserDao")),
        "{diagnostics:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("query method `UserDao.body` must be abstract")),
        "{diagnostics:?}"
    );
    assert!(
        messages.iter().any(|message| {
            message.contains("query method `UserDao.namedParam` only supports required positional")
        }),
        "{diagnostics:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("query method `UserDao.dynamicSql` SQL must be")),
        "{diagnostics:?}"
    );
    assert!(
        messages.iter().any(|message| message
            .contains("query method `UserDao.badReturn` must return Future<Result<T, SqlxError>>")),
        "{diagnostics:?}"
    );
}

#[test]
fn validates_duplicate_effective_columns() {
    let mut class = row_class();
    class.fields[1].configs = Vec::new();
    class.fields[1].name = "id".to_owned();
    let diagnostics = register_plugin().validate(&library(vec![class]));

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("duplicate SQL column `id`")),
        "{diagnostics:?}"
    );
}

#[test]
fn validates_skipped_field_requires_default() {
    let mut class = row_class();
    class.fields = vec![field(
        "sessionActive",
        TypeIr::bool(),
        vec![config("dust_dart::Sqlx", "(skip: true)")],
    )];
    class.constructors[0].params = vec![named_param("sessionActive", TypeIr::bool(), false)];
    let diagnostics = register_plugin().validate(&library(vec![class]));

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("uses `Sqlx(skip: true)` without a default")),
        "{diagnostics:?}"
    );
}
