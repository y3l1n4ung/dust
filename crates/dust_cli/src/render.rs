use dust_diagnostics::{Diagnostic, Severity, render_to_string};
use dust_driver::CommandResult;

use crate::args::CliCommand;

const BANNER: &str = include_str!("../../../assets/dust-logo-cli.txt");

fn render_banner() -> &'static str {
    BANNER.trim_end()
}

pub(crate) fn render_result(command: &CliCommand, result: &CommandResult) -> String {
    let mut lines = Vec::new();
    lines.push(render_banner().to_owned());
    lines.push(String::new());

    match command {
        CliCommand::Build => {
            let generated = result
                .build_artifacts
                .iter()
                .filter(|artifact| artifact.written)
                .count();
            let total = result.build_artifacts.len();
            let skipped = total.saturating_sub(generated);
            lines.push(format!(
                "build  scanned: {total}  generated: {generated}  skipped: {skipped}  time: {}ms",
                result.elapsed_ms
            ));
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
            }
        }
        CliCommand::Watch => {
            let generated = result
                .build_artifacts
                .iter()
                .filter(|artifact| artifact.written)
                .count();
            let total = result.build_artifacts.len();
            let skipped = total.saturating_sub(generated);
            lines.push(format!(
                "watch  scanned: {total}  generated: {generated}  skipped: {skipped}  time: {}ms",
                result.elapsed_ms
            ));
            if let Some(watch) = &result.watch {
                lines.push(format!(
                    "watch  cycles: {}  rebuilds: {}",
                    watch.cycles, watch.rebuild_batches
                ));
            }
        }
    }

    if !result.diagnostics.is_empty() {
        lines.push(render_diagnostic_summary(&result.diagnostics));
        lines.push(String::new());
        append_diagnostic_blocks(&mut lines, &result.diagnostics);
    }

    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}

fn render_diagnostic_summary(diagnostics: &[Diagnostic]) -> String {
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Warning)
        .count();
    let notes = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Note)
        .count();

    format!("diagnostics  errors: {errors}  warnings: {warnings}  notes: {notes}")
}

fn append_diagnostic_blocks(lines: &mut Vec<String>, diagnostics: &[Diagnostic]) {
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        lines.extend(render_to_string(diagnostic).lines().map(str::to_owned));
        if index + 1 != diagnostics.len() {
            lines.push(String::new());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use dust_diagnostics::{Diagnostic, SourceLabel};
    use dust_driver::{CacheReport, CleanReport, CommandResult, DoctorReport, WatchReport};
    use dust_text::{FileId, TextRange};

    use super::*;

    #[test]
    fn render_diagnostic_summary_counts_each_severity() {
        let summary = render_diagnostic_summary(&[
            Diagnostic::error("broken"),
            Diagnostic::warning("suspicious"),
            Diagnostic::note("try again"),
        ]);

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
        );
        let doctor = render_result(
            &CliCommand::Doctor,
            &CommandResult {
                doctor: Some(DoctorReport {
                    package_root: PathBuf::from("/tmp/project"),
                    package_config_path: PathBuf::from(
                        "/tmp/workspace/.dart_tool/package_config.json",
                    ),
                    library_count: 7,
                    plugin_names: vec!["derive".to_owned(), "serde".to_owned()],
                    libraries: vec![PathBuf::from("lib/user.dart")],
                }),
                elapsed_ms: 9,
                ..CommandResult::default()
            },
        );

        assert!(clean.contains("clean  scanned: 4  removed: 3  cache: cleared  time: 18ms"));
        assert!(doctor.contains("doctor  workspace: ok  libraries: 7  plugins: 2  time: 9ms"));
        assert!(doctor.contains("plugins derive, serde"));
        assert!(doctor.contains("package /tmp/project"));
        assert!(doctor.contains("config  /tmp/workspace/.dart_tool/package_config.json"));
    }

    #[test]
    fn render_watch_and_diagnostics() {
        let rendered = render_result(
            &CliCommand::Watch,
            &CommandResult {
                watch: Some(WatchReport {
                    cycles: 2,
                    rebuild_batches: 1,
                    rebuilt_libraries: vec![PathBuf::from("lib/user.dart")],
                }),
                cache: Some(CacheReport::default()),
                elapsed_ms: 22,
                diagnostics: vec![Diagnostic::warning("something happened").with_label(
                    SourceLabel::new(
                        FileId::new(4),
                        TextRange::new(22_u32, 27_u32),
                        "this annotation name is not registered",
                    ),
                )],
                ..CommandResult::default()
            },
        );

        assert!(rendered.contains("watch  scanned: 0  generated: 0  skipped: 0  time: 22ms"));
        assert!(rendered.contains("watch  cycles: 2  rebuilds: 1"));
        assert!(rendered.contains("diagnostics  errors: 0  warnings: 1  notes: 0"));
        assert!(rendered.contains("warning: something happened"));
        assert!(rendered.contains("file FileId(4) 22..27"));
    }
}
