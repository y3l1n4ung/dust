use dust_http_client_plugin::register_plugin;
use dust_plugin_api::DustPlugin;

#[test]
fn plugin_claims_http_client_configs() {
    let plugin = register_plugin();
    let claimed = plugin.claimed_configs();

    assert!(claimed.contains(&"dust_http_client_annotation::HttpClient"));
    assert!(claimed.contains(&"dust_http_client_annotation::GET"));
    assert!(claimed.contains(&"dust_http_client_annotation::Body"));
    assert!(claimed.contains(&"dust_http_client_annotation::HttpParse"));
    assert!(claimed.contains(&"dust_http_client_annotation::GenerateTest"));
}
