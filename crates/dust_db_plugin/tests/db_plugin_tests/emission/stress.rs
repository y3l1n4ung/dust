use dust_db_plugin::register_plugin;
use dust_ir::{MethodIr, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};

use crate::support::*;

#[test]
fn emits_many_dao_queries_with_static_mapper_references() {
    let mut dao = dao_class();
    dao.methods = (0..100)
        .map(|index| MethodIr {
            name: format!("findById{index}"),
            is_static: false,
            is_external: false,
            return_type: result_type(TypeIr::named("UserProfile").nullable()),
            has_body: false,
            body_source: None,
            params: vec![method_param("id", TypeIr::int())],
            span: span(),
            traits: Vec::new(),
            configs: vec![config(
                "dust_dart::Query",
                &format!("(r'SELECT id, display_name FROM users WHERE id = $1 /* {index} */')"),
            )],
        })
        .collect();

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
    let source = &contribution.support_types[0];

    assert!(source.contains("Future<Result<UserProfile?, SqlxError>> findById99(int id)"));
    assert!(source.contains("const $UserProfileRowDeserializer().deserialize"));
    assert!(!source.contains("(row) =>"));
}
