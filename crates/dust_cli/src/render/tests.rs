use std::path::PathBuf;

use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_driver::{
    CacheReport, CleanReport, CommandResult, DiagnosticFile, DoctorPackageCompatibility,
    DoctorPackageCompatibilityStatus, DoctorReport, RouteInspectionEntry, RouteTableReport,
    WatchReport,
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
                package_config_path: PathBuf::from("/tmp/workspace/.dart_tool/package_config.json"),
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
        diagnostics: vec![
            Diagnostic::warning("something happened").with_label(SourceLabel::new(
                FileId::new(4),
                TextRange::new(22_u32, 27_u32),
                "this annotation name is not registered",
            )),
        ],
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
fn render_route_table_summary_and_routes() {
    let rendered = render_result(
        &CliCommand::RouteTable,
        &CommandResult {
            route_table: Some(RouteTableReport {
                scanned_files: 3,
                routes: vec![
                    RouteInspectionEntry {
                        name: "dashboard".to_owned(),
                        path: "/dashboard".to_owned(),
                        page: "DashboardPage".to_owned(),
                        shell: Some("AppShell".to_owned()),
                        branch: Some("mainTabs".to_owned()),
                        guards: Vec::new(),
                        requires_auth: true,
                        result_type: "void".to_owned(),
                    },
                    RouteInspectionEntry {
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
