use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use dust_diagnostics::Diagnostic;
use dust_parser_dart_ts::scan_i18n_source;
use dust_text::{FileId, SourceText};
use dust_workspace::{detect_workspace_root, load_dust_config};

use crate::{
    request::I18nScanRequest,
    result::{CommandResult, DiagnosticFile, I18nScanEntry, I18nScanReport},
};

/// Runs a static i18n source scan over `lib/**/*.dart`.
pub fn run_i18n_scan(request: I18nScanRequest) -> CommandResult {
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

    match scan_workspace(
        &package_root,
        &dust_config.outputs.primary_suffix,
        &mut result,
    ) {
        Ok(report) => result.i18n_scan = Some(report),
        Err(diagnostic) => result.diagnostics.push(diagnostic),
    }

    result.elapsed_ms = started.elapsed().as_millis();
    result
}

/// Scans package source files and returns a report.
fn scan_workspace(
    package_root: &Path,
    primary_suffix: &str,
    result: &mut CommandResult,
) -> Result<I18nScanReport, Diagnostic> {
    let lib_dir = package_root.join("lib");
    if !lib_dir.is_dir() {
        return Ok(I18nScanReport::default());
    }

    let mut paths = Vec::new();
    collect_dart_files(&lib_dir, primary_suffix, &mut paths)?;
    paths.sort();

    let mut report = I18nScanReport {
        scanned_files: paths.len(),
        entries: Vec::new(),
    };
    for (index, path) in paths.into_iter().enumerate() {
        scan_file(FileId::new(index as u32 + 1), &path, result, &mut report)?;
    }
    report.entries = unique_entries(report.entries);
    Ok(report)
}

/// Merges duplicate static keys into one deterministic entry.
fn unique_entries(entries: Vec<I18nScanEntry>) -> Vec<I18nScanEntry> {
    let mut unique = BTreeMap::<String, I18nScanEntry>::new();
    for entry in entries {
        unique
            .entry(entry.key.clone())
            .and_modify(|existing| merge_entry(existing, &entry))
            .or_insert(entry);
    }
    unique.into_values().collect()
}

/// Merges placeholder/default metadata from a repeated key use.
fn merge_entry(existing: &mut I18nScanEntry, incoming: &I18nScanEntry) {
    if existing.default_text.is_none() {
        existing.default_text.clone_from(&incoming.default_text);
    }
    for arg in &incoming.args {
        if !existing.args.contains(arg) {
            existing.args.push(arg.clone());
        }
    }
}

/// Scans one Dart source file into the report.
fn scan_file(
    file_id: FileId,
    path: &Path,
    result: &mut CommandResult,
    report: &mut I18nScanReport,
) -> Result<(), Diagnostic> {
    let source = fs::read_to_string(path).map_err(|error| {
        Diagnostic::error(format!("failed to read `{}`: {error}", path.display()))
    })?;
    let source = Arc::<str>::from(source);
    let source_text = SourceText::new(file_id, Arc::clone(&source));
    let scanned = scan_i18n_source(&source_text);
    let has_labels = scanned.diagnostics.iter().any(Diagnostic::has_labels);

    for entry in scanned.entries {
        report.entries.push(I18nScanEntry {
            key: entry.key,
            namespace: entry.namespace,
            default_text: entry.default_text,
            args: entry.args,
        });
    }

    result.diagnostics.extend(scanned.diagnostics);
    if has_labels {
        result
            .diagnostic_files
            .push(DiagnosticFile::new(file_id, path.to_path_buf(), source));
    }
    Ok(())
}

/// Recursively collects source Dart files under one directory.
fn collect_dart_files(
    dir: &Path,
    primary_suffix: &str,
    out: &mut Vec<PathBuf>,
) -> Result<(), Diagnostic> {
    let entries = fs::read_dir(dir).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read directory `{}`: {error}",
            dir.display()
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            Diagnostic::error(format!(
                "failed to enumerate directory `{}`: {error}",
                dir.display()
            ))
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            Diagnostic::error(format!(
                "failed to inspect directory entry `{}`: {error}",
                path.display()
            ))
        })?;

        if file_type.is_dir() {
            collect_dart_files(&path, primary_suffix, out)?;
        } else if file_type.is_file() && should_scan_file(&path, primary_suffix) {
            out.push(path);
        }
    }

    Ok(())
}

/// Returns whether one file is app source for i18n scanning.
fn should_scan_file(path: &Path, primary_suffix: &str) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    name.ends_with(".dart") && !name.ends_with(primary_suffix) && !name.ends_with(".g.dart")
}
