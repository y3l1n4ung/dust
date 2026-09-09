use dust_diagnostics::{Diagnostic, Severity, render_to_string_with_files};
use dust_driver::CommandResult;

use crate::args::CliCommand;

/// Embedded ASCII banner shown before command summaries.
const BANNER: &str = include_str!("../assets/dust-logo-cli.txt");

/// Returns the banner without trailing asset newlines.
fn render_banner() -> &'static str {
    BANNER.trim_end()
}

/// Renders a complete command result for stdout or stderr.
pub(crate) fn render_result(command: &CliCommand, result: &CommandResult, ai_mode: bool) -> String {
    let mut lines = Vec::new();
    if !ai_mode {
        lines.push(render_banner().to_owned());
        lines.push(String::new());
    }

    match command {
        CliCommand::Build => {
            append_generation_summary(&mut lines, "build", result);
        }
        CliCommand::DbBuild => {
            append_generation_summary(&mut lines, "db build", result);
        }
        CliCommand::Clean => {
            if let Some(clean) = &result.clean {
                let cache = if clean.cache_cleared {
                    "cleared"
                } else {
                    "none"
                };
                lines.push(format!(
                    "clean  scanned: {}  removed: {}  cache: {cache}  time: {}ms",
                    clean.scanned_files, clean.removed_files, result.elapsed_ms
                ));
            }
        }
        CliCommand::Check => {
            let stale = result
                .checked_libraries
                .iter()
                .filter(|library| library.stale)
                .count();
            let total = result.checked_libraries.len();
            let fresh = total.saturating_sub(stale);
            lines.push(format!(
                "check  scanned: {total}  clean: {fresh}  stale: {stale}  time: {}ms",
                result.elapsed_ms
            ));
        }
        CliCommand::Doctor => {
            if let Some(doctor) = &result.doctor {
                let status = if result.has_errors() { "issues" } else { "ok" };
                lines.push(format!(
                    "doctor  workspace: {status}  libraries: {}  plugins: {}  time: {}ms",
                    doctor.library_count,
                    doctor.plugin_names.len(),
                    result.elapsed_ms
                ));
                if !doctor.plugin_names.is_empty() {
                    lines.push(format!("plugins {}", doctor.plugin_names.join(", ")));
                }
                lines.push(format!("package {}", doctor.package_root.display()));
                lines.push(format!("config  {}", doctor.package_config_path.display()));
                lines.push(format!("compat cli {}", doctor.cli_version));
                for package in &doctor.package_compatibility {
                    let usage = if package.used_by_workspace {
                        "used"
                    } else {
                        "unused"
                    };
                    let resolved = package.resolved_version.as_deref().unwrap_or("-");
                    let supported = package.supported_constraint.as_deref().unwrap_or("-");
                    lines.push(format!(
                        "compat {} status={} usage={} resolved={} supported={}",
                        package.package_name,
                        package.status.as_str(),
                        usage,
                        resolved,
                        supported
                    ));
                    if let Some(action) = &package.action {
                        lines.push(format!("compat {} action={}", package.package_name, action));
                    }
                }
            }
        }
        CliCommand::RouteTable => {
            if let Some(table) = &result.route_table {
                lines.push(format!(
                    "route table  scanned: {}  routes: {}  time: {}ms",
                    table.scanned_files,
                    table.routes.len(),
                    result.elapsed_ms
                ));
                if !table.routes.is_empty() {
                    append_route_table(&mut lines, &table.routes);
                }
            }
        }
        CliCommand::I18nBuild => {
            if let Some(build) = &result.i18n_build {
                lines.push(format!(
                    "i18n build  files: {}  changed: {}  keys: {}  added: {}  synced: {}  preview: {}  time: {}ms",
                    build.arb_files,
                    build.changed_files,
                    build.keys,
                    build.added_messages,
                    build.synced_messages,
                    build.dry_run,
                    result.elapsed_ms
                ));
            }
        }
        CliCommand::I18nCheck => {
            if let Some(check) = &result.i18n_check {
                lines.push(format!(
                    "i18n check  files: {}  keys: {}  checked: {}  stale: {}  time: {}ms",
                    check.arb_files,
                    check.keys,
                    check.checked_messages,
                    check.stale_messages,
                    result.elapsed_ms
                ));
            }
        }
        CliCommand::I18nScan => {
            if let Some(scan) = &result.i18n_scan {
                lines.push(format!(
                    "i18n scan  files: {}  keys: {}  time: {}ms",
                    scan.scanned_files,
                    scan.entries.len(),
                    result.elapsed_ms
                ));
                for entry in &scan.entries {
                    let args = if entry.args.is_empty() {
                        "-".to_owned()
                    } else {
                        entry.args.join(",")
                    };
                    let default = entry
                        .default_text
                        .as_ref()
                        .map_or_else(|| "-".to_owned(), |text| format!("{text:?}"));
                    lines.push(format!(
                        "{}  namespace={}  default={}  args={}",
                        entry.key, entry.namespace, default, args
                    ));
                }
            }
        }
        CliCommand::Watch => {
            append_generation_summary(&mut lines, "watch", result);
            if let Some(watch) = &result.watch {
                lines.push(format!(
                    "watch  cycles: {}  rebuilds: {}",
                    watch.cycles, watch.rebuild_batches
                ));
            }
        }
        CliCommand::Upgrade => {}
    }

    if !result.diagnostics.is_empty() {
        lines.push(render_diagnostic_summary(&result.diagnostics));
        lines.push(String::new());
        append_diagnostic_blocks(&mut lines, result, &result.diagnostics);
    }

    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}

/// Appends a stable route table.
fn append_route_table(lines: &mut Vec<String>, routes: &[dust_driver::RouteInspectionEntry]) {
    lines.push("name | path | page | shell | branch | guards | auth | result".to_owned());
    lines.push("--- | --- | --- | --- | --- | --- | --- | ---".to_owned());
    for route in routes {
        lines.push(format!(
            "{} | {} | {} | {} | {} | {} | {} | {}",
            route.name,
            route.path,
            route.page,
            route.shell.as_deref().unwrap_or("-"),
            route.branch.as_deref().unwrap_or("-"),
            if route.guards.is_empty() {
                "-".to_owned()
            } else {
                route.guards.join(",")
            },
            route_auth_label(route.requires_auth),
            route.result_type,
        ));
    }
}

/// Renders the generated auth state for one route.
fn route_auth_label(requires_auth: bool) -> &'static str {
    if requires_auth { "protected" } else { "public" }
}

/// Appends the build/check/watch artifact summary line.
fn append_generation_summary(lines: &mut Vec<String>, label: &str, result: &CommandResult) {
    let written = result
        .build_artifacts
        .iter()
        .filter(|artifact| artifact.written)
        .count();
    let routed = result
        .build_artifacts
        .iter()
        .filter(|artifact| !artifact.written && artifact.routed)
        .count();
    let cached = result
        .build_artifacts
        .iter()
        .filter(|artifact| !artifact.written && !artifact.routed && artifact.cached)
        .count();
    let generated = written + routed;
    let total = result.build_artifacts.len();
    let skipped = total.saturating_sub(generated + cached);
    lines.push(format!(
        "{label}  scanned: {total}  generated: {generated}  cached: {cached}  skipped: {skipped}  time: {}ms",
        result.elapsed_ms
    ));
}

/// Renders the aggregate diagnostic severity count line.
fn render_diagnostic_summary(diagnostics: &[Diagnostic]) -> String {
    let counts = DiagnosticCounts::from_diagnostics(diagnostics);

    format!(
        "diagnostics  errors: {}  warnings: {}  notes: {}",
        counts.errors, counts.warnings, counts.notes
    )
}

/// Severity counts used by CLI diagnostic summaries.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct DiagnosticCounts {
    /// Number of error diagnostics.
    errors: usize,
    /// Number of warning diagnostics.
    warnings: usize,
    /// Number of note diagnostics.
    notes: usize,
}

impl DiagnosticCounts {
    /// Counts diagnostics by severity.
    fn from_diagnostics(diagnostics: &[Diagnostic]) -> Self {
        let mut counts = Self::default();
        for diagnostic in diagnostics {
            match diagnostic.severity {
                Severity::Error => counts.errors += 1,
                Severity::Warning => counts.warnings += 1,
                Severity::Note => counts.notes += 1,
            }
        }
        counts
    }
}

/// Appends rendered diagnostic blocks with source context.
fn append_diagnostic_blocks(
    lines: &mut Vec<String>,
    result: &CommandResult,
    diagnostics: &[Diagnostic],
) {
    let files = result
        .diagnostic_files
        .iter()
        .map(|file| file.render_context())
        .collect::<Vec<_>>();
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        lines.extend(
            render_to_string_with_files(diagnostic, &files)
                .lines()
                .map(str::to_owned),
        );
        if index + 1 != diagnostics.len() {
            lines.push(String::new());
        }
    }
}

#[cfg(test)]
#[path = "render/tests.rs"]
mod tests;
