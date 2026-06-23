use dust_diagnostics::Diagnostic;
use dust_ir::DartFileIr;
use dust_parser_dart::ParsedDartFileSurface;
use dust_plugin_api::{
    DustPlugin, PluginContribution, SymbolPlan, WorkspaceAnalysisBuilder, WorkspaceAnalysisContext,
};

use crate::{
    analysis::collect_workspace_analysis,
    emit::emit_library,
    features::{COPY_WITH_SYMBOL, DEBUG_SYMBOL, EQ_SYMBOL, TO_STRING_SYMBOL, VALIDATE_SYMBOL},
    validate::validate_library,
};

/// The built-in plugin that implements Dust's core derive traits.
pub struct DerivePlugin;

/// Creates the built-in derive plugin instance.
pub fn register_plugin() -> DerivePlugin {
    DerivePlugin
}

/// Trait symbols claimed by the derive plugin.
const CLAIMED_TRAITS: &[&str] = &[
    TO_STRING_SYMBOL,
    DEBUG_SYMBOL,
    EQ_SYMBOL,
    COPY_WITH_SYMBOL,
    VALIDATE_SYMBOL,
];

/// Config symbols claimed by the derive plugin.
const CLAIMED_CONFIGS: &[&str] = &[VALIDATE_SYMBOL];

/// Short annotation names supported by the derive plugin.
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

    fn requested_symbols(&self, _library: &DartFileIr) -> Vec<String> {
        Vec::new()
    }

    fn collect_workspace_analysis(
        &self,
        _context: WorkspaceAnalysisContext<'_>,
        library: &ParsedDartFileSurface,
        analysis: &mut WorkspaceAnalysisBuilder,
    ) {
        collect_workspace_analysis(library, analysis);
    }

    fn validate(&self, library: &DartFileIr) -> Vec<Diagnostic> {
        validate_library(library)
    }

    fn emit(&self, library: &DartFileIr, plan: &SymbolPlan) -> PluginContribution {
        emit_library(library, plan)
    }
}
