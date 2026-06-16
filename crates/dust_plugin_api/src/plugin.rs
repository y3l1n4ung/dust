use std::path::Path;

use dust_diagnostics::Diagnostic;
use dust_ir::DartFileIr;
use dust_parser_dart::ParsedDartFileSurface;

use crate::{PluginContribution, SymbolPlan, WorkspaceAnalysisBuilder};

/// Context available to future generated-unit based plugin APIs.
#[derive(Debug, Clone, Copy)]
pub struct PluginContext<'a> {
    /// The deterministic symbol plan for the current file.
    pub symbol_plan: &'a SymbolPlan,
}

/// Compatibility generated-unit type while plugins still return one merged contribution.
pub type GeneratedUnit = PluginContribution;

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

    /// Validates the Dart file from this plugin's point of view.
    fn validate(&self, file: &DartFileIr) -> Vec<Diagnostic>;

    /// Produces generated fragments for this plugin.
    fn emit(&self, file: &DartFileIr, plan: &SymbolPlan) -> PluginContribution;

    /// Produces generated units for the future plugin API.
    fn generate(&self, file: &DartFileIr, context: &PluginContext<'_>) -> Vec<GeneratedUnit> {
        vec![self.emit(file, context.symbol_plan)]
    }
}
