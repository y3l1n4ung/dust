use dust_plugin_api::DustPlugin;
use dust_route_plugin::register_plugin;

#[test]
fn registers_route_annotation_symbols() {
    let plugin = register_plugin();

    assert_eq!(plugin.plugin_name(), "Route");
    assert_eq!(
        plugin.claimed_configs(),
        vec![
            "dust_flutter::Router",
            "dust_flutter::Route",
            "dust_flutter::GeneratedRoute"
        ]
    );
    assert_eq!(
        plugin.supported_annotations(),
        vec!["Router", "Route", "GeneratedRoute"]
    );
}
