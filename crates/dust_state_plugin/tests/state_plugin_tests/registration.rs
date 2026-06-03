use dust_plugin_api::DustPlugin;
use dust_state_plugin::register_plugin;

use super::support::library_with_classes;

#[test]
fn plugin_claims_view_model_config() {
    let plugin = register_plugin();

    assert_eq!(plugin.plugin_name(), "ViewModel");
    assert_eq!(plugin.claimed_configs(), vec!["dust_flutter::ViewModel"]);
    assert_eq!(plugin.supported_annotations(), vec!["ViewModel"]);
}

#[test]
fn plugin_contract_returns_empty_output_for_unannotated_library() {
    let plugin = dust_state_plugin::StatePlugin::new();
    let library = library_with_classes(Vec::new());

    assert!(plugin.validate(&library).is_empty());
    assert_eq!(
        plugin.emit(&library, &dust_plugin_api::SymbolPlan::default()),
        dust_plugin_api::PluginContribution::default()
    );
}
