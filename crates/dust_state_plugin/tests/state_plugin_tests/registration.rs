use dust_plugin_api::DustPlugin;
use dust_state_plugin::register_plugin;

#[test]
fn plugin_claims_view_model_config() {
    let plugin = register_plugin();

    assert_eq!(plugin.claimed_configs(), vec!["dust_state::ViewModel"]);
    assert_eq!(plugin.supported_annotations(), vec!["ViewModel"]);
}
