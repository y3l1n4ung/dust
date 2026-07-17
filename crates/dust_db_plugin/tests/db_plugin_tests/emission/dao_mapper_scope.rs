use dust_db_plugin::register_plugin;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use crate::support::*;

#[test]
fn generated_dao_uses_direct_from_row_mapper() {
    let contribution = register_plugin()
        .generate(
            &library(vec![simple_user_row_class(), dao_class()]),
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    let source = &contribution.support_types[0];
    assert!(source.contains("UserProfileFromRow.fromRow"));
    assert!(!source.contains("RowMapperRegistry.map"));
    assert!(!source.contains("queryAs<"));
}
