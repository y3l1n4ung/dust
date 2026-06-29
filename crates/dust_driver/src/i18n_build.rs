use std::time::Instant;

use dust_diagnostics::Diagnostic;
use dust_workspace::{detect_workspace_root, load_dust_config};

use crate::{
    i18n_arb::write_i18n_arb_files, i18n_bootstrap::build_i18n_bootstrap,
    i18n_scan::scan_workspace_sources, request::I18nBuildRequest, result::CommandResult,
};

/// Runs i18n scan, ARB reconciliation, and bootstrap generation.
pub fn run_i18n_build(request: I18nBuildRequest) -> CommandResult {
    let started = Instant::now();
    let mut result = CommandResult::default();

    let package_root = match detect_workspace_root(&request.cwd) {
        Ok(root) => root,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };
    let dust_config = match load_dust_config(&package_root) {
        Ok(config) => config,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };
    let Some(i18n_config) = &dust_config.i18n else {
        result.diagnostics.push(Diagnostic::error(
            "dust i18n build requires i18n.locales in dust.yaml",
        ));
        result.elapsed_ms = started.elapsed().as_millis();
        return result;
    };

    let scan = match scan_workspace_sources(
        &package_root,
        &dust_config.outputs.primary_suffix,
        &mut result,
    ) {
        Ok(report) => report,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };

    match write_i18n_arb_files(
        &package_root,
        i18n_config,
        scan.scanned_files,
        &scan.entries,
    ) {
        Ok(report) => result.i18n_build = Some(report),
        Err(diagnostic) => result.diagnostics.push(diagnostic),
    }
    result.i18n_scan = Some(scan);

    if !result.has_errors() {
        match build_i18n_bootstrap(&package_root, &dust_config) {
            Ok(Some(artifact)) => result.build_artifacts.push(artifact),
            Ok(None) => {}
            Err(diagnostic) => result.diagnostics.push(diagnostic),
        }
    }

    result.elapsed_ms = started.elapsed().as_millis();
    result
}
