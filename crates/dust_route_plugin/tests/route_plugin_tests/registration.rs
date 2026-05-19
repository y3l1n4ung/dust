use dust_plugin_api::DustPlugin;
use dust_route_plugin::register_plugin;

#[test]
fn registers_route_annotation_symbols() {
    let plugin = register_plugin();

    assert_eq!(plugin.plugin_name(), "Route");
    assert!(plugin.claimed_configs().contains(&"dust_router::Route"));
    assert!(plugin.claimed_configs().contains(&"dust_router::Router"));
    assert!(plugin.supported_annotations().contains(&"Route"));
    assert!(plugin.supported_annotations().contains(&"Router"));
}
