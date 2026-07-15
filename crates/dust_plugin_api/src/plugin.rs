use std::path::Path;

use dust_diagnostics::Diagnostic;
use dust_ir::DartFileIr;
use dust_parser_dart::ParsedDartFileSurface;

use crate::{PluginContribution, SymbolPlan, WorkspaceAnalysisBuilder};

/// Context available while generating plugin output.
#[derive(Debug, Clone, Copy)]
pub struct PluginContext<'a> {
    /// The deterministic symbol plan for the current file.
    pub symbol_plan: &'a SymbolPlan,
}

/// Source context available while collecting parse-only workspace facts.
#[derive(Debug, Clone, Copy)]
pub struct WorkspaceAnalysisContext<'a> {
    /// The package name for the library being scanned.
    pub package_name: &'a str,
    /// The package root directory for the library being scanned.
    pub package_root: &'a Path,
    /// The source file path for the library being scanned.
    pub source_path: &'a Path,
}

/// The contract implemented by every Dust generation plugin.
pub trait DustPlugin: Send + Sync {
    /// Returns the stable plugin name used in diagnostics and reports.
    fn plugin_name(&self) -> &'static str;

    /// Returns the fully-qualified trait symbols this plugin exclusively owns.
    fn claimed_traits(&self) -> &'static [&'static str] {
        &[]
    }

    /// Returns the fully-qualified config symbols this plugin exclusively owns.
    fn claimed_configs(&self) -> &'static [&'static str] {
        &[]
    }

    /// Returns config symbols that do not require the source library to declare a generated part.
    fn partless_configs(&self) -> &'static [&'static str] {
        &[]
    }

    /// Returns the surface-level annotation names this plugin handles.
    ///
    /// These names are used during the fast-path discovery phase to identify
    /// candidate libraries before full parsing or resolution.
    fn supported_annotations(&self) -> &'static [&'static str] {
        &[]
    }

    /// Returns generated helper symbol names this plugin wants reserved.
    fn requested_symbols(&self, _file: &DartFileIr) -> Vec<String> {
        Vec::new()
    }

    /// Collects parse-only workspace facts for this plugin during the shared scan phase.
    fn collect_workspace_analysis(
        &self,
        _context: WorkspaceAnalysisContext<'_>,
        _file: &ParsedDartFileSurface,
        _analysis: &mut WorkspaceAnalysisBuilder,
    ) {
    }

    /// Collects workspace facts from canonical IR during the staged plugin migration.
    ///
    /// The parser-surface method remains available as a compatibility adapter until
    /// every plugin has moved to this IR-only hook.
    fn collect_workspace_analysis_ir(
        &self,
        _file: &DartFileIr,
        _analysis: &mut WorkspaceAnalysisBuilder,
    ) {
    }

    /// Validates the Dart file from this plugin's point of view.
    fn validate(&self, file: &DartFileIr) -> Vec<Diagnostic>;

    /// Validates the Dart file with access to the current generated symbol plan.
    fn validate_with_plan(&self, file: &DartFileIr, _plan: &SymbolPlan) -> Vec<Diagnostic> {
        self.validate(file)
    }

    /// Produces generated contributions for this plugin.
    fn generate(&self, file: &DartFileIr, context: &PluginContext<'_>) -> Vec<PluginContribution>;
}
