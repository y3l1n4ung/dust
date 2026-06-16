#[path = "support/library.rs"]
mod library;

use dust_diagnostics::Diagnostic;
use dust_ir::DartFileIr;
use dust_plugin_api::{DustPlugin, PluginContext, PluginContribution, PluginRegistry, SymbolPlan};

use self::library::sample_library;

#[test]
fn registry_runs_validation_and_emission_in_registration_order() {
    struct OrderPlugin {
        name: &'static str,
    }

    impl DustPlugin for OrderPlugin {
        fn plugin_name(&self) -> &'static str {
            self.name
        }

        fn validate(&self, _library: &DartFileIr) -> Vec<Diagnostic> {
            vec![Diagnostic::note(format!("validated by {}", self.name))]
        }

        fn emit(&self, _library: &DartFileIr, _plan: &SymbolPlan) -> PluginContribution {
            let mut contribution = PluginContribution::default();
            contribution.push_mixin_member("User", format!("// {}", self.name));
            contribution
        }
    }

    let library = sample_library();
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(OrderPlugin { name: "a" }))
        .unwrap();
    registry
        .register(Box::new(OrderPlugin { name: "b" }))
        .unwrap();

    let diagnostics = registry.validate_library(&library);
    let plan = registry.build_symbol_plan(&library);
    let contributions = registry.emit_contributions(&library, &plan);

    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>(),
        vec!["validated by a", "validated by b"]
    );
    assert_eq!(contributions.len(), 2);
    assert_eq!(contributions[0].mixin_members[0].members[0], "// a");
    assert_eq!(contributions[1].mixin_members[0].members[0], "// b");
}

#[test]
fn plugin_generate_defaults_to_legacy_contribution_adapter() {
    struct GeneratePlugin;

    impl DustPlugin for GeneratePlugin {
        fn plugin_name(&self) -> &'static str {
            "generate"
        }

        fn validate(&self, _file: &DartFileIr) -> Vec<Diagnostic> {
            Vec::new()
        }

        fn emit(&self, _file: &DartFileIr, _plan: &SymbolPlan) -> PluginContribution {
            let mut contribution = PluginContribution::default();
            contribution.push_mixin_member("User", "// generated");
            contribution
        }
    }

    let file = sample_library();
    let plan = SymbolPlan::default();
    let units = GeneratePlugin.generate(&file, &PluginContext { symbol_plan: &plan });

    assert_eq!(units.len(), 1);
    assert_eq!(units[0].mixin_members[0].members[0], "// generated");
}
