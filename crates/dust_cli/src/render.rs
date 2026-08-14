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
        CliCommand::RouteGraph => {
            if let Some(graph) = &result.route_graph {
                lines.push(format!(
                    "route graph  scanned: {}  routes: {}  time: {}ms",
                    graph.scanned_files,
                    graph.nodes.len(),
                    result.elapsed_ms
                ));
                if !graph.nodes.is_empty() {
                    append_route_graph(&mut lines, &graph.nodes);
                }
            }
        }
        CliCommand::RouteFixtures => {
            if let Some(fixtures) = &result.route_fixtures {
                lines.push(format!(
                    "route fixtures  scanned: {}  fixtures: {}  time: {}ms",
                    fixtures.scanned_files,
                    fixtures.fixtures.len(),
                    result.elapsed_ms
                ));
                if !fixtures.fixtures.is_empty() {
                    append_route_fixtures(&mut lines, &fixtures.fixtures);
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

/// Appends stable deep-link fixture rows as a Markdown table.
fn append_route_fixtures(lines: &mut Vec<String>, fixtures: &[dust_driver::RouteFixtureRow]) {
    lines.push("route | case | valid | shape | uri | expected".to_owned());
    lines.push("--- | --- | --- | --- | --- | ---".to_owned());
    for fixture in fixtures {
        lines.push(format!(
            "{} | {} | {} | {} | {} | {}",
            fixture.route,
            fixture.case_name,
            fixture.valid,
            fixture.shape,
            fixture.uri,
            fixture.expected
        ));
    }
}

/// Appends a stable route graph as Markdown table rows.
fn append_route_graph(lines: &mut Vec<String>, nodes: &[dust_driver::RouteGraphNode]) {
    lines.push("path | parent | name | page | shell | branch | guards".to_owned());
    lines.push("--- | --- | --- | --- | --- | --- | ---".to_owned());
    for node in nodes {
        lines.push(format!(
            "{} | {} | {} | {} | {} | {} | {}",
            node.path,
            node.parent_path.as_deref().unwrap_or("-"),
            node.name,
            node.page,
            node.shell.as_deref().unwrap_or("-"),
            node.branch.as_deref().unwrap_or("-"),
            if node.guards.is_empty() {
                "-".to_owned()
            } else {
                node.guards.join(",")
            },
        ));
    }
}

/// Appends a stable route table.
fn append_route_table(lines: &mut Vec<String>, routes: &[dust_driver::RouteTableRow]) {
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
mod tests {
    use std::path::PathBuf;

    use dust_diagnostics::{Diagnostic, SourceLabel};
    use dust_driver::{
        CacheReport, CleanReport, CommandResult, DiagnosticFile, DoctorPackageCompatibility,
        DoctorPackageCompatibilityStatus, DoctorReport, RouteFixtureRow, RouteFixturesReport,
        RouteGraphNode, RouteGraphReport, RouteTableReport, RouteTableRow, WatchReport,
    };
    use dust_text::{FileId, TextRange};

    use super::*;

    #[test]
    fn render_diagnostic_summary_counts_each_severity() {
        let diagnostics = [
            Diagnostic::error("broken"),
            Diagnostic::warning("suspicious"),
            Diagnostic::note("try again"),
        ];
        let counts = DiagnosticCounts::from_diagnostics(&diagnostics);
        let summary = render_diagnostic_summary(&[
            Diagnostic::error("broken"),
            Diagnostic::warning("suspicious"),
            Diagnostic::note("try again"),
        ]);

        assert_eq!(
            counts,
            DiagnosticCounts {
                errors: 1,
                warnings: 1,
                notes: 1,
            }
        );
        assert_eq!(summary, "diagnostics  errors: 1  warnings: 1  notes: 1");
    }

    #[test]
    fn render_clean_and_doctor_summaries() {
        let clean = render_result(
            &CliCommand::Clean,
            &CommandResult {
                clean: Some(CleanReport {
                    package_root: PathBuf::from("/tmp/project"),
                    scanned_files: 4,
                    removed_files: 3,
                    cache_cleared: true,
                }),
                elapsed_ms: 18,
                ..CommandResult::default()
            },
            false,
        );
        let doctor = render_result(
            &CliCommand::Doctor,
            &CommandResult {
                doctor: Some(DoctorReport {
                    cli_version: "0.1.4".to_owned(),
                    package_root: PathBuf::from("/tmp/project"),
                    package_config_path: PathBuf::from(
                        "/tmp/workspace/.dart_tool/package_config.json",
                    ),
                    library_count: 7,
                    plugin_names: vec!["derive".to_owned(), "serde".to_owned()],
                    libraries: vec![PathBuf::from("lib/user.dart")],
                    package_compatibility: vec![DoctorPackageCompatibility {
                        package_name: "dust_dart".to_owned(),
                        used_by_workspace: true,
                        resolved_version: Some("0.1.3".to_owned()),
                        supported_constraint: Some(">=0.1.3 <0.2.0".to_owned()),
                        status: DoctorPackageCompatibilityStatus::Compatible,
                        action: None,
                    }],
                }),
                elapsed_ms: 9,
                ..CommandResult::default()
            },
            false,
        );

        assert!(clean.contains("clean  scanned: 4  removed: 3  cache: cleared  time: 18ms"));
        assert!(doctor.contains("doctor  workspace: ok  libraries: 7  plugins: 2  time: 9ms"));
        assert!(doctor.contains("plugins derive, serde"));
        assert!(doctor.contains("package /tmp/project"));
        assert!(doctor.contains("config  /tmp/workspace/.dart_tool/package_config.json"));
        assert!(doctor.contains("compat cli 0.1.4"));
        assert!(doctor.contains(
            "compat dust_dart status=compatible usage=used resolved=0.1.3 supported=>=0.1.3 <0.2.0"
        ));
    }

    #[test]
    fn render_watch_and_diagnostics() {
        let result = CommandResult {
            watch: Some(WatchReport {
                cycles: 2,
                rebuild_batches: 1,
                rebuilt_libraries: vec![PathBuf::from("lib/user.dart")],
            }),
            cache: Some(CacheReport::default()),
            diagnostic_files: vec![DiagnosticFile::new(
                FileId::new(4),
                PathBuf::from("/tmp/example/user.dart"),
                "@Derive([ToString(), UnknownTrait()])\n",
            )],
            elapsed_ms: 22,
            diagnostics: vec![Diagnostic::warning("something happened").with_label(
                SourceLabel::new(
                    FileId::new(4),
                    TextRange::new(22_u32, 27_u32),
                    "this annotation name is not registered",
                ),
            )],
            ..CommandResult::default()
        };
        let rendered = render_result(&CliCommand::Watch, &result, false);
        let compact = render_result(&CliCommand::Watch, &result, true);

        assert!(
            rendered.contains("watch  scanned: 0  generated: 0  cached: 0  skipped: 0  time: 22ms")
        );
        assert!(rendered.contains("watch  cycles: 2  rebuilds: 1"));
        assert!(rendered.contains("diagnostics  errors: 0  warnings: 1  notes: 0"));
        assert!(rendered.contains("warning: something happened"));
        assert!(rendered.contains("  --> /tmp/example/user.dart:1:23"));
        assert!(rendered.contains("1 | @Derive([ToString(), UnknownTrait()])"));
        assert!(rendered.contains("^^^^^ this annotation name is not registered"));
        assert!(!compact.contains(render_banner()));
        assert!(compact.contains("diagnostics  errors: 0  warnings: 1  notes: 0"));
        assert!(compact.contains("warning: something happened"));
    }

    #[test]
    fn render_route_table_summary_and_rows() {
        let rendered = render_result(
            &CliCommand::RouteTable,
            &CommandResult {
                route_table: Some(RouteTableReport {
                    scanned_files: 3,
                    routes: vec![
                        RouteTableRow {
                            name: "dashboard".to_owned(),
                            path: "/dashboard".to_owned(),
                            page: "DashboardPage".to_owned(),
                            shell: Some("AppShell".to_owned()),
                            branch: Some("mainTabs".to_owned()),
                            guards: Vec::new(),
                            requires_auth: true,
                            result_type: "void".to_owned(),
                        },
                        RouteTableRow {
                            name: "checkout".to_owned(),
                            path: "/checkout".to_owned(),
                            page: "CheckoutPage".to_owned(),
                            shell: None,
                            branch: None,
                            guards: vec!["CartGuard".to_owned()],
                            requires_auth: true,
                            result_type: "bool".to_owned(),
                        },
                    ],
                }),
                elapsed_ms: 12,
                ..CommandResult::default()
            },
            true,
        );

        assert_eq!(
            rendered,
            "route table  scanned: 3  routes: 2  time: 12ms\n\
             name | path | page | shell | branch | guards | auth | result\n\
             --- | --- | --- | --- | --- | --- | --- | ---\n\
             dashboard | /dashboard | DashboardPage | AppShell | mainTabs | - | protected | void\n\
             checkout | /checkout | CheckoutPage | - | - | CartGuard | protected | bool\n"
        );
    }

    #[test]
    fn render_route_graph_summary_and_nodes() {
        let rendered = render_result(
            &CliCommand::RouteGraph,
            &CommandResult {
                route_graph: Some(RouteGraphReport {
                    scanned_files: 3,
                    nodes: vec![
                        RouteGraphNode {
                            name: "dashboard".to_owned(),
                            path: "/dashboard".to_owned(),
                            parent_path: None,
                            page: "DashboardPage".to_owned(),
                            shell: Some("AppShell".to_owned()),
                            branch: Some("mainTabs".to_owned()),
                            guards: Vec::new(),
                        },
                        RouteGraphNode {
                            name: "orders".to_owned(),
                            path: "/dashboard/orders".to_owned(),
                            parent_path: Some("/dashboard".to_owned()),
                            page: "OrdersPage".to_owned(),
                            shell: Some("AppShell".to_owned()),
                            branch: Some("mainTabs".to_owned()),
                            guards: vec!["OrderGuard".to_owned()],
                        },
                    ],
                }),
                elapsed_ms: 12,
                ..CommandResult::default()
            },
            true,
        );

        assert_eq!(
            rendered,
            "route graph  scanned: 3  routes: 2  time: 12ms\n\
             path | parent | name | page | shell | branch | guards\n\
             --- | --- | --- | --- | --- | --- | ---\n\
             /dashboard | - | dashboard | DashboardPage | AppShell | mainTabs | -\n\
             /dashboard/orders | /dashboard | orders | OrdersPage | AppShell | mainTabs | OrderGuard\n"
        );
    }

    #[test]
    fn render_route_fixtures_summary_and_rows() {
        let rendered = render_result(
            &CliCommand::RouteFixtures,
            &CommandResult {
                route_fixtures: Some(RouteFixturesReport {
                    scanned_files: 3,
                    fixtures: vec![
                        RouteFixtureRow {
                            route: "product".to_owned(),
                            case_name: "path".to_owned(),
                            valid: true,
                            shape: "path".to_owned(),
                            uri: "/products/42?tab=reviews".to_owned(),
                            expected: "typed-route".to_owned(),
                        },
                        RouteFixtureRow {
                            route: "product".to_owned(),
                            case_name: "invalid-path-param".to_owned(),
                            valid: false,
                            shape: "path".to_owned(),
                            uri: "/products/not-an-int?tab=reviews".to_owned(),
                            expected: "not-found-route".to_owned(),
                        },
                    ],
                }),
                elapsed_ms: 12,
                ..CommandResult::default()
            },
            true,
        );

        assert_eq!(
            rendered,
            "route fixtures  scanned: 3  fixtures: 2  time: 12ms\n\
             route | case | valid | shape | uri | expected\n\
             --- | --- | --- | --- | --- | ---\n\
             product | path | true | path | /products/42?tab=reviews | typed-route\n\
             product | invalid-path-param | false | path | /products/not-an-int?tab=reviews | not-found-route\n"
        );
    }

    #[test]
    fn ai_mode_omits_banner_but_keeps_summary() {
        let normal = render_result(
            &CliCommand::Build,
            &CommandResult {
                elapsed_ms: 7,
                ..CommandResult::default()
            },
            false,
        );
        let compact = render_result(
            &CliCommand::Build,
            &CommandResult {
                elapsed_ms: 7,
                ..CommandResult::default()
            },
            true,
        );

        assert!(normal.starts_with(render_banner()));
        assert!(!compact.contains(render_banner()));
        assert_eq!(
            compact,
            "build  scanned: 0  generated: 0  cached: 0  skipped: 0  time: 7ms\n"
        );
    }
}
