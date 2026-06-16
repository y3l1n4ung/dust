#[path = "support/library.rs"]
mod library;

use dust_diagnostics::Diagnostic;
use dust_ir::LibraryIr;
use dust_parser_dart::ParsedLibrarySurface;
use dust_plugin_api::{
    DustPlugin, PluginContribution, PluginRegistry, SymbolPlan, WorkspaceAnalysisBuilder,
    WorkspaceAnalysisContext,
};
use dust_text::TextRange;

use self::library::sample_library;

struct SymbolPlugin {
    name: &'static str,
    requested: Vec<&'static str>,
}

impl DustPlugin for SymbolPlugin {
    fn plugin_name(&self) -> &'static str {
        self.name
    }

    fn requested_symbols(&self, _library: &LibraryIr) -> Vec<String> {
        self.requested
            .iter()
            .map(|name| (*name).to_owned())
            .collect()
    }

    fn validate(&self, _library: &LibraryIr) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn emit(&self, _library: &LibraryIr, _plan: &SymbolPlan) -> PluginContribution {
        PluginContribution::default()
    }
}

fn sample_parsed_library() -> ParsedLibrarySurface {
    ParsedLibrarySurface {
        span: TextRange::new(0_u32, 100_u32),
        directives: Vec::new(),
        classes: Vec::new(),
        enums: Vec::new(),
        mixins: Vec::new(),
        extensions: Vec::new(),
        extension_types: Vec::new(),
        functions: Vec::new(),
        variables: Vec::new(),
        typedefs: Vec::new(),
        query_calls: Vec::new(),
    }
}

#[test]
fn symbol_plan_preserves_first_seen_order_and_dedupes() {
    let library = sample_library();
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(SymbolPlugin {
            name: "plugin_a",
            requested: vec!["_$UserToJson", "_undefined"],
        }))
        .unwrap();
    registry
        .register(Box::new(SymbolPlugin {
            name: "plugin_b",
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
            _context: WorkspaceAnalysisContext<'_>,
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
    registry.collect_workspace_analysis(
        WorkspaceAnalysisContext {
            package_name: "test_app",
            package_root: std::path::Path::new("."),
            source_path: std::path::Path::new("lib/test.dart"),
        },
        &sample_parsed_library(),
        &mut analysis,
    );
    let analysis = analysis.build();

    assert_eq!(
        analysis.string_set("a"),
        Some(&["Team".to_owned(), "User".to_owned()][..])
    );
}
