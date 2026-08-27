use std::sync::Arc;

use dust_diagnostics::Diagnostic;
use dust_ir::DartFileIr;
use dust_plugin_api::{DustPlugin, SymbolPlan, WorkspaceAnalysisBuilder};

/// Builds the symbol plan a package's libraries are validated against.
///
/// The driver scans every library for workspace facts before it validates any
/// of them. A test that validates one library in isolation sees no schema and
/// no row classes, which is the bug this exists to keep from coming back.
pub(crate) fn package_plan(plugin: &impl DustPlugin, libraries: &[&DartFileIr]) -> SymbolPlan {
    let mut analysis = WorkspaceAnalysisBuilder::default();
    for library in libraries {
        plugin.collect_workspace_analysis_ir(library, &mut analysis);
    }
    let mut plan = SymbolPlan::default();
    plan.set_workspace_analysis(Arc::new(analysis.build()));
    plan
}

/// Validates `target` against the facts collected from every library in `libraries`.
pub(crate) fn validate_in_package(
    plugin: &impl DustPlugin,
    libraries: &[&DartFileIr],
    target: &DartFileIr,
) -> Vec<Diagnostic> {
    plugin.validate_with_plan(target, &package_plan(plugin, libraries))
}

/// Validates one library that is the whole package.
pub(crate) fn validate_alone(plugin: &impl DustPlugin, library: &DartFileIr) -> Vec<Diagnostic> {
    validate_in_package(plugin, &[library], library)
}
