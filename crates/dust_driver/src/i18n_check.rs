use std::{collections::BTreeMap, path::Path, time::Instant};

use dust_diagnostics::Diagnostic;
use dust_workspace::{I18nConfig, detect_workspace_root, load_dust_config};

use self::{
    read::{fallback_messages, namespaces_to_check, read_namespace_files},
    validate::{
        expected_entries, validate_expected_messages, validate_locale_marker,
        validate_stale_messages,
    },
};
use crate::{
    i18n_keys::{I18nPlannedEntry, plan_i18n_entries},
    i18n_scan::scan_workspace_sources,
    request::I18nCheckRequest,
    result::{CommandResult, I18nCheckReport},
};

/// ARB file loading and namespace discovery.
mod read;
/// ARB validation rules.
mod validate;

/// Runs i18n scan and no-write ARB validation.
pub fn run_i18n_check(request: I18nCheckRequest) -> CommandResult {
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
            "dust i18n check requires i18n.locales in dust.yaml",
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

    match check_i18n_arb_files(
        &package_root,
        i18n_config,
        scan.scanned_files,
        &scan.entries,
    ) {
        Ok((report, diagnostics)) => {
            result.i18n_check = Some(report);
            result.diagnostics.extend(diagnostics);
        }
        Err(diagnostic) => result.diagnostics.push(diagnostic),
    }
    result.i18n_scan = Some(scan);

    result.elapsed_ms = started.elapsed().as_millis();
    result
}

/// Checks configured ARB assets against scanned static translation entries.
fn check_i18n_arb_files(
    package_root: &Path,
    config: &I18nConfig,
    scanned_files: usize,
    entries: &[crate::result::I18nScanEntry],
) -> Result<(I18nCheckReport, Vec<Diagnostic>), Diagnostic> {
    let planned = plan_i18n_entries(entries)?;
    let by_namespace = entries_by_namespace(&planned);
    let namespaces = namespaces_to_check(package_root, config, &by_namespace)?;
    let mut report = I18nCheckReport {
        scanned_files,
        keys: planned.len(),
        ..I18nCheckReport::default()
    };
    let mut diagnostics = Vec::new();

    for namespace in namespaces {
        let entries = by_namespace
            .get(&namespace)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        check_namespace(
            package_root,
            config,
            &namespace,
            entries,
            &mut report,
            &mut diagnostics,
        );
    }

    Ok((report, diagnostics))
}

/// Returns planned entries keyed by namespace.
fn entries_by_namespace(entries: &[I18nPlannedEntry]) -> BTreeMap<String, Vec<I18nPlannedEntry>> {
    let mut by_namespace = BTreeMap::<String, Vec<I18nPlannedEntry>>::new();
    for entry in entries {
        by_namespace
            .entry(entry.namespace.clone())
            .or_default()
            .push(entry.clone());
    }
    by_namespace
}

/// Checks one namespace across all configured locales.
fn check_namespace(
    package_root: &Path,
    config: &I18nConfig,
    namespace: &str,
    entries: &[I18nPlannedEntry],
    report: &mut I18nCheckReport,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let files = read_namespace_files(package_root, config, namespace, report, diagnostics);
    let fallback = fallback_messages(config, &files);
    let expected = expected_entries(entries);

    for file in &files {
        if file.arb.is_none() {
            continue;
        }
        validate_locale_marker(file, diagnostics);
        validate_expected_messages(
            file,
            &expected,
            fallback.as_ref(),
            config,
            report,
            diagnostics,
        );
        validate_stale_messages(namespace, file, &expected, report, diagnostics);
    }
}
