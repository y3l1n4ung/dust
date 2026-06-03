use dust_http_client_plugin::register_plugin;
use dust_plugin_api::DustPlugin;

#[test]
fn plugin_claims_http_client_configs() {
    let plugin = register_plugin();
    let claimed = plugin.claimed_configs();

    assert!(claimed.contains(&"dust_dart::HttpClient"));
    assert!(claimed.contains(&"dust_dart::GET"));
    assert!(claimed.contains(&"dust_dart::Body"));
    assert!(claimed.contains(&"dust_dart::HttpParse"));
    assert!(claimed.contains(&"dust_dart::GenerateTest"));
}
