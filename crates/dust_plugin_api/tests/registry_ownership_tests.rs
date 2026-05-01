mod support;

use dust_ir::SymbolId;
use dust_plugin_api::PluginRegistry;

use self::support::FakePlugin;

#[test]
fn registry_rejects_duplicate_trait_ownership() {
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "plugin_a",
            traits: vec![SymbolId::new("derive_annotation::ToString")],
            configs: Vec::new(),
            requested: Vec::new(),
        }))
        .unwrap();

    let error = registry
        .register(Box::new(FakePlugin {
            name: "plugin_b",
            traits: vec![SymbolId::new("derive_annotation::ToString")],
            configs: Vec::new(),
            requested: Vec::new(),
        }))
        .unwrap_err();

    assert!(
        error
            .message
            .contains("trait symbol `derive_annotation::ToString` is already owned")
    );
}

#[test]
fn registry_rejects_duplicate_config_ownership() {
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "plugin_a",
            traits: Vec::new(),
            configs: vec![SymbolId::new("derive_serde_annotation::SerDe")],
            requested: Vec::new(),
        }))
        .unwrap();

    let error = registry
        .register(Box::new(FakePlugin {
            name: "plugin_b",
            traits: Vec::new(),
            configs: vec![SymbolId::new("derive_serde_annotation::SerDe")],
            requested: Vec::new(),
        }))
        .unwrap_err();

    assert!(
        error
            .message
            .contains("config symbol `derive_serde_annotation::SerDe` is already owned")
    );
}

#[test]
fn plugin_names_follow_registration_order() {
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "derive",
            traits: Vec::new(),
            configs: Vec::new(),
            requested: Vec::new(),
        }))
        .unwrap();
    registry
        .register(Box::new(FakePlugin {
            name: "serde",
            traits: Vec::new(),
            configs: Vec::new(),
            requested: Vec::new(),
        }))
        .unwrap();

    assert_eq!(registry.plugin_names(), vec!["derive", "serde"]);
}
