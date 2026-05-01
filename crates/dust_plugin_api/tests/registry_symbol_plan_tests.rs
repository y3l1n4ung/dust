mod support;

use dust_parser_dart::ParsedLibrarySurface;
use dust_plugin_api::{
    DustPlugin, PluginContribution, PluginRegistry, SymbolPlan, WorkspaceAnalysisBuilder,
};
use dust_text::TextRange;

use self::support::{FakePlugin, sample_library};

fn sample_parsed_library() -> ParsedLibrarySurface {
    ParsedLibrarySurface {
        span: TextRange::new(0_u32, 100_u32),
        directives: Vec::new(),
        classes: Vec::new(),
        enums: Vec::new(),
    }
}

#[test]
fn symbol_plan_preserves_first_seen_order_and_dedupes() {
    let library = sample_library();
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "plugin_a",
            traits: Vec::new(),
            configs: Vec::new(),
            requested: vec!["_$UserToJson", "_undefined"],
        }))
        .unwrap();
    registry
        .register(Box::new(FakePlugin {
            name: "plugin_b",
            traits: Vec::new(),
            configs: Vec::new(),
            requested: vec!["_undefined", "_$UserFromJson"],
        }))
        .unwrap();

    let plan = registry.build_symbol_plan(&library);

    let names = plan
        .reserved()
        .iter()
        .map(|symbol| symbol.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["_$UserToJson", "_undefined", "_$UserFromJson"]);
    assert!(plan.contains("_undefined"));
}

#[test]
fn registry_collects_workspace_analysis_in_registration_order() {
    struct AnalysisPlugin {
        key: &'static str,
        value: &'static str,
    }

    impl DustPlugin for AnalysisPlugin {
        fn plugin_name(&self) -> &'static str {
            self.key
        }

        fn collect_workspace_analysis(
            &self,
            _library: &ParsedLibrarySurface,
            analysis: &mut WorkspaceAnalysisBuilder,
        ) {
            analysis.add_string_set_value(self.key, self.value);
        }

        fn validate(&self, _library: &dust_ir::LibraryIr) -> Vec<dust_diagnostics::Diagnostic> {
            Vec::new()
        }

        fn emit(&self, _library: &dust_ir::LibraryIr, _plan: &SymbolPlan) -> PluginContribution {
            PluginContribution::default()
        }
    }

    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(AnalysisPlugin {
            key: "a",
            value: "User",
        }))
        .unwrap();
    registry
        .register(Box::new(AnalysisPlugin {
            key: "a",
            value: "Team",
        }))
        .unwrap();

    let mut analysis = WorkspaceAnalysisBuilder::default();
    registry.collect_workspace_analysis(&sample_parsed_library(), &mut analysis);
    let analysis = analysis.build();

    assert_eq!(
        analysis.string_set("a"),
        Some(&["Team".to_owned(), "User".to_owned()][..])
    );
}
