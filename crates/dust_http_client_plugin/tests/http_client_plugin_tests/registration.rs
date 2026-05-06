use dust_http_client_plugin::register_plugin;
use dust_plugin_api::DustPlugin;

#[test]
fn plugin_claims_http_client_configs() {
    let plugin = register_plugin();
    let claimed = plugin.claimed_configs();
    let names = claimed
        .iter()
        .map(|symbol| symbol.0.as_str())
        .collect::<Vec<_>>();

    assert!(names.contains(&"dust_http_client_annotation::HttpClient"));
    assert!(names.contains(&"dust_http_client_annotation::GET"));
    assert!(names.contains(&"dust_http_client_annotation::Body"));
    assert!(names.contains(&"dust_http_client_annotation::HttpParse"));
    assert!(names.contains(&"dust_http_client_annotation::GenerateTest"));
}
