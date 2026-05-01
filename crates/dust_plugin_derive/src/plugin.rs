use dust_diagnostics::Diagnostic;
use dust_ir::{LibraryIr, SymbolId};
use dust_parser_dart::ParsedLibrarySurface;
use dust_plugin_api::{DustPlugin, PluginContribution, SymbolPlan, WorkspaceAnalysisBuilder};

use crate::{
    analysis::collect_workspace_analysis, emit::emit_library,
    features::clone_copy_with::copy_with_requires_undefined, validate::validate_library,
};

/// The built-in plugin that implements Dust's core derive traits.
pub struct DerivePlugin;

/// Creates the built-in derive plugin instance.
pub fn register_plugin() -> DerivePlugin {
    DerivePlugin
}

impl DustPlugin for DerivePlugin {
    fn plugin_name(&self) -> &'static str {
        "dust_plugin_derive"
    }

    fn claimed_traits(&self) -> Vec<SymbolId> {
        vec![
            SymbolId::new("derive_annotation::ToString"),
            SymbolId::new("derive_annotation::Debug"),
            SymbolId::new("derive_annotation::Eq"),
            SymbolId::new("derive_annotation::CopyWith"),
        ]
    }

    fn requested_symbols(&self, library: &LibraryIr) -> Vec<String> {
        let needs_undefined = library.classes.iter().any(|class| {
            class.traits.iter().any(|trait_app| {
                trait_app.symbol.0 == "derive_annotation::CopyWith"
                    && copy_with_requires_undefined(class)
            })
        });

        if needs_undefined {
            vec!["_undefined".to_owned()]
        } else {
            Vec::new()
        }
    }

    fn collect_workspace_analysis(
        &self,
        library: &ParsedLibrarySurface,
        analysis: &mut WorkspaceAnalysisBuilder,
    ) {
        collect_workspace_analysis(library, analysis);
    }

    fn validate(&self, library: &LibraryIr) -> Vec<Diagnostic> {
        validate_library(library)
    }

    fn emit(&self, library: &LibraryIr, plan: &SymbolPlan) -> PluginContribution {
        emit_library(library, plan)
    }
}
