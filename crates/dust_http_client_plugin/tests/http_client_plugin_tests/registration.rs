use dust_http_client_plugin::register_plugin;
use dust_ir::TraitApplicationIr;
use dust_plugin_api::{DustPlugin, WorkspaceAnalysisBuilder};

use super::support::{field, library_with_classes, serde_model_class, span};

#[test]
fn plugin_claims_http_client_configs() {
    let plugin = register_plugin();
    let claimed = plugin.claimed_configs();

    assert!(claimed.contains(&"dust_dart::HttpClient"));
    assert!(claimed.contains(&"dust_dart::GET"));
    assert!(claimed.contains(&"dust_dart::Body"));
    assert!(claimed.contains(&"dust_dart::HttpParse"));
    assert!(!claimed.contains(&"dust_dart::GenerateTest"));
}

#[test]
fn collects_json_workspace_facts_from_canonical_ir() {
    let mut model = serde_model_class("User", vec![field("id", dust_ir::TypeIr::string())]);
    model.traits.push(TraitApplicationIr {
        symbol: dust_ir::SymbolId::new("dust_dart::Serialize"),
        span: span(2, 3),
    });
    let library = library_with_classes(vec![model]);
    let mut analysis = WorkspaceAnalysisBuilder::default();

    register_plugin().collect_workspace_analysis_ir(&library, &mut analysis);

    let snapshot = analysis.snapshot();
    assert_eq!(
        snapshot.string_set("dust_http_client_plugin.json_types.v1"),
        Some(&["User".to_owned()][..])
    );
    assert_eq!(
        snapshot.string_set("dust_http_client_plugin.json_serializable_types.v1"),
        Some(&["User".to_owned()][..])
    );
    assert_eq!(
        snapshot.string_set("dust_http_client_plugin.json_from_json_types.v1"),
        Some(&["User".to_owned()][..])
    );
}
