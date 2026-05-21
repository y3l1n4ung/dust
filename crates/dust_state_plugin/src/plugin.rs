use dust_diagnostics::Diagnostic;
use dust_ir::LibraryIr;
use dust_parser_dart::ParsedLibrarySurface;
use dust_plugin_api::{
    DustPlugin, PluginContribution, SymbolPlan, WorkspaceAnalysisBuilder, WorkspaceAnalysisContext,
};

mod analysis;
mod constants;
mod emit;
mod model;
mod parse;
mod validate;

use self::analysis::collect_state_workspace_analysis;
use self::constants::{CLAIMED_CONFIG_SYMBOLS, SUPPORTED_ANNOTATIONS};
use self::emit::emit_library_state;
use self::validate::validate_library_state;

/// Dust plugin for typed Flutter state management.
pub struct StatePlugin;

impl StatePlugin {
    /// Creates a new state plugin.
    pub fn new() -> Self {
        Self
    }
}

impl Default for StatePlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates the state plugin.
pub fn register_plugin() -> StatePlugin {
    StatePlugin::new()
}

impl DustPlugin for StatePlugin {
    fn plugin_name(&self) -> &'static str {
        "ViewModel"
    }

    fn claimed_configs(&self) -> &'static [&'static str] {
        CLAIMED_CONFIG_SYMBOLS
    }

    fn supported_annotations(&self) -> &'static [&'static str] {
        SUPPORTED_ANNOTATIONS
    }

    fn collect_workspace_analysis(
        &self,
        context: WorkspaceAnalysisContext<'_>,
        library: &ParsedLibrarySurface,
        analysis: &mut WorkspaceAnalysisBuilder,
    ) {
        collect_state_workspace_analysis(context, library, analysis);
    }

    fn validate(&self, library: &LibraryIr) -> Vec<Diagnostic> {
        validate_library_state(library)
    }

    fn emit(&self, library: &LibraryIr, plan: &SymbolPlan) -> PluginContribution {
        emit_library_state(library, plan)
    }
}
