use std::fs;

use dust_db_plugin::register_plugin;
use dust_ir::{MethodIr, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};

use crate::support::*;

/// The Dart a DAO is expected to emit, one fixture per shape.
#[path = "dao/expected.rs"]
mod expected;
use self::expected::*;

#[test]
fn emits_sqlx_style_dao_redirecting_factory_impl() {
    let plugin = register_plugin();
    let contribution = plugin
        .generate(
            &library(vec![row_class(), dao_class()]),
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert_eq!(contribution.support_types[0], expected_default_dao_output());
}

#[test]
fn emits_driver_method_for_each_query_cardinality() {
    let mut dao = dao_class();
    dao.methods = vec![
        dao_method(
            "findRequired",
            result_type(TypeIr::named("UserProfile")),
            vec![method_param("id", TypeIr::int())],
            "(r'SELECT id, display_name FROM users WHERE id = $1')",
        ),
        dao_method(
            "list",
            result_type(TypeIr::generic("List", vec![TypeIr::named("UserProfile")])),
            Vec::new(),
            "(r'SELECT id, display_name FROM users')",
        ),
        dao_method(
            "rawRows",
            result_type(TypeIr::generic("List", vec![TypeIr::named("Row")])),
            Vec::new(),
            "(r'SELECT id, display_name FROM users')",
        ),
        dao_method(
            "deleteAll",
            result_type(TypeIr::named("Unit")),
            Vec::new(),
            "(r'DELETE FROM users')",
        ),
    ];

    let contribution = register_plugin()
        .generate(
            &library(vec![row_class(), dao]),
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert_eq!(contribution.support_types[0], expected_cardinality_output());
}

#[test]
fn emits_dao_mapper_for_imported_from_row_return_type() {
    let root = temp_root("imported_dao_rows");
    fs::create_dir_all(root.join("lib/models")).unwrap();
    fs::write(
        root.join("lib/models/user_profile.dart"),
        "@Derive([FromRow()])\nfinal class UserProfile {}\n",
    )
    .unwrap();

    let plugin = register_plugin();
    let contribution = plugin
        .generate(
            &library_with_imports(
                &root,
                "lib/dao/user_dao.dart",
                vec!["../models/user_profile.dart"],
                vec![dao_class()],
            ),
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert_eq!(
        contribution.support_types[0],
        expected_imported_dao_output()
    );

    let _ = fs::remove_dir_all(root);
}

/// Emitted SQL keeps `$n` and binds arguments in declaration order.
///
/// Rewriting here would have to pick a dialect: Postgres takes `$n` unchanged
/// and SQLite does not. The driver rewrites at bind time, which is also what
/// makes a `@SqlxDao` method and an inline query agree about the same text.
#[test]
fn emits_sql_verbatim_so_the_driver_owns_the_placeholder_form() {
    let mut dao = dao_class();
    dao.methods = vec![dao_method(
        "findForOrg",
        result_type(TypeIr::named("UserProfile").nullable()),
        vec![
            method_param("id", TypeIr::int()),
            method_param("orgId", TypeIr::int()),
        ],
        "(r'SELECT id, display_name FROM users WHERE org_id = $2 OR id = $1 OR backup_id = $1')",
    )];

    let contribution = register_plugin()
        .generate(
            &library(vec![simple_user_row_class(), dao]),
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert_eq!(
        contribution.support_types[0],
        expected_reordered_sqlite_output()
    );
}

fn dao_method(
    name: &str,
    return_type: TypeIr,
    params: Vec<dust_ir::MethodParamIr>,
    query: &str,
) -> MethodIr {
    MethodIr {
        name: name.to_owned(),
        is_static: false,
        is_external: false,
        return_type,
        has_body: false,
        body_source: None,
        params,
        span: span(),
        traits: Vec::new(),
        configs: vec![config("dust_dart::Query", query)],
    }
}
