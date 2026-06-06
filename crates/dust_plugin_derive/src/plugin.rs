use dust_diagnostics::Diagnostic;
use dust_ir::LibraryIr;
use dust_parser_dart::ParsedLibrarySurface;
use dust_plugin_api::{
    DustPlugin, PluginContribution, SymbolPlan, WorkspaceAnalysisBuilder, WorkspaceAnalysisContext,
};

use crate::{
    analysis::collect_workspace_analysis,
    emit::emit_library,
    features::{
        COPY_WITH_SYMBOL, DEBUG_SYMBOL, EQ_SYMBOL, TO_STRING_SYMBOL, VALIDATE_SYMBOL,
        clone_copy_with::copy_with_requires_undefined,
    },
    validate::validate_library,
};

/// The built-in plugin that implements Dust's core derive traits.
pub struct DerivePlugin;

/// Creates the built-in derive plugin instance.
pub fn register_plugin() -> DerivePlugin {
    DerivePlugin
}

const CLAIMED_TRAITS: &[&str] = &[
    TO_STRING_SYMBOL,
    DEBUG_SYMBOL,
    EQ_SYMBOL,
    COPY_WITH_SYMBOL,
    VALIDATE_SYMBOL,
];

const CLAIMED_CONFIGS: &[&str] = &[VALIDATE_SYMBOL];

const SUPPORTED_ANNOTATIONS: &[&str] =
    &["Derive", "ToString", "Debug", "Eq", "CopyWith", "Validate"];

impl DustPlugin for DerivePlugin {
    fn plugin_name(&self) -> &'static str {
        "dust_plugin_derive"
    }

    fn claimed_traits(&self) -> &'static [&'static str] {
        CLAIMED_TRAITS
    }

    fn claimed_configs(&self) -> &'static [&'static str] {
        CLAIMED_CONFIGS
    }

    fn supported_annotations(&self) -> &'static [&'static str] {
        SUPPORTED_ANNOTATIONS
    }

    fn requested_symbols(&self, library: &LibraryIr) -> Vec<String> {
        let needs_undefined = library.classes.iter().any(|class| {
            class.traits.iter().any(|trait_app| {
                trait_app.symbol.0 == COPY_WITH_SYMBOL && copy_with_requires_undefined(class)
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
        _context: WorkspaceAnalysisContext<'_>,
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
