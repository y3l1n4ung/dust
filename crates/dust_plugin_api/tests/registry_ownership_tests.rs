//! Integration tests for plugin registry symbol ownership validation.

use dust_diagnostics::Diagnostic;
use dust_ir::DartFileIr;
use dust_plugin_api::{DustPlugin, PluginContext, PluginContribution, PluginRegistry};

struct FakePlugin {
    name: &'static str,
    traits: &'static [&'static str],
    configs: &'static [&'static str],
}

impl DustPlugin for FakePlugin {
    fn plugin_name(&self) -> &'static str {
        self.name
    }

    fn claimed_traits(&self) -> &'static [&'static str] {
        self.traits
    }

    fn claimed_configs(&self) -> &'static [&'static str] {
        self.configs
    }

    fn validate(&self, _library: &DartFileIr) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn generate(
        &self,
        _library: &DartFileIr,
        _context: &PluginContext<'_>,
    ) -> Vec<PluginContribution> {
        vec![PluginContribution::default()]
    }
}

#[test]
fn registry_rejects_duplicate_trait_ownership() {
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "plugin_a",
            traits: &["dust_dart::ToString"],
            configs: &[],
        }))
        .unwrap();

    let error = registry
        .register(Box::new(FakePlugin {
            name: "plugin_b",
            traits: &["dust_dart::ToString"],
            configs: &[],
        }))
        .unwrap_err();

    assert!(
        error
            .message
            .contains("trait symbol `dust_dart::ToString` is already owned")
    );
}

#[test]
fn registry_rejects_duplicate_config_ownership() {
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "plugin_a",
            traits: &[],
            configs: &["dust_dart::SerDe"],
        }))
        .unwrap();

    let error = registry
        .register(Box::new(FakePlugin {
            name: "plugin_b",
            traits: &[],
            configs: &["dust_dart::SerDe"],
        }))
        .unwrap_err();

    assert!(
        error
            .message
            .contains("config symbol `dust_dart::SerDe` is already owned")
    );
}

#[test]
fn plugin_names_follow_registration_order() {
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "derive",
            traits: &[],
            configs: &[],
        }))
        .unwrap();
    registry
        .register(Box::new(FakePlugin {
            name: "serde",
            traits: &[],
            configs: &[],
        }))
        .unwrap();

    assert_eq!(registry.plugin_names(), vec!["derive", "serde"]);
}
