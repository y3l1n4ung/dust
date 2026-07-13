//! Integration tests for plugin registry validation and emission execution.

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

        fn generate(
            &self,
            _library: &DartFileIr,
            _context: &PluginContext<'_>,
        ) -> Vec<PluginContribution> {
            let mut contribution = PluginContribution::default();
            contribution.push_mixin_member("User", format!("// {}", self.name));
            vec![contribution]
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
    let contributions = registry.generate_units(&library, &plan);

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
fn plugin_generate_returns_contribution_units() {
    struct GeneratePlugin;

    impl DustPlugin for GeneratePlugin {
        fn plugin_name(&self) -> &'static str {
            "generate"
        }

        fn validate(&self, _file: &DartFileIr) -> Vec<Diagnostic> {
            Vec::new()
        }

        fn generate(
            &self,
            _file: &DartFileIr,
            _context: &PluginContext<'_>,
        ) -> Vec<PluginContribution> {
            let mut contribution = PluginContribution::default();
            contribution.push_mixin_member("User", "// generated");
            vec![contribution]
        }
    }

    let file = sample_library();
    let plan = SymbolPlan::default();
    let units = GeneratePlugin.generate(&file, &PluginContext { symbol_plan: &plan });

    assert_eq!(units.len(), 1);
    assert_eq!(units[0].mixin_members[0].members[0], "// generated");
}

#[test]
fn registry_emits_generated_units_from_generate_api() {
    struct GenerateOnlyPlugin;

    impl DustPlugin for GenerateOnlyPlugin {
        fn plugin_name(&self) -> &'static str {
            "generate-only"
        }

        fn validate(&self, _file: &DartFileIr) -> Vec<Diagnostic> {
            Vec::new()
        }

        fn generate(
            &self,
            _file: &DartFileIr,
            context: &PluginContext<'_>,
        ) -> Vec<PluginContribution> {
            let mut first = PluginContribution::default();
            first.push_mixin_member("User", "// first");

            let mut second = PluginContribution::default();
            second.top_level_functions.push(format!(
                "// symbols: {}",
                context.symbol_plan.reserved().len()
            ));

            vec![first, second]
        }
    }

    let file = sample_library();
    let plan = SymbolPlan::default();
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(GenerateOnlyPlugin)).unwrap();

    let contributions = registry.generate_units(&file, &plan);

    assert_eq!(contributions.len(), 2);
    assert_eq!(contributions[0].mixin_members[0].members[0], "// first");
    assert_eq!(contributions[1].top_level_functions[0], "// symbols: 0");
}
