use dust_driver::CommandResult;

use crate::args::CliCommand;

const BANNER: &str = include_str!("../../../assets/dust-logo-cli.txt");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn render_banner() -> &'static str {
    BANNER.trim_end()
}

pub(crate) fn render_help() -> String {
    [
        &format!("dust {VERSION}"),
        "",
        "Commands:",
        "  build     Scan Dart sources and generate `.g.dart` outputs",
        "  watch     Keep running and regenerate affected outputs",
        "  clean     Remove Dust-generated outputs and cache",
        "  help      Show this help",
        "",
        "Options:",
        "  --root <path>       Use a specific package root",
        "  --fail-fast         Stop after the first error",
        "  --poll-ms <ms>      Watch poll interval in milliseconds",
        "  --max-cycles <n>    Stop watch after n cycles",
        "  -h, --help          Show this help",
    ]
    .join("\n")
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
        CliCommand::Help => lines.push(render_help()),
    }

    for diagnostic in &result.diagnostics {
        lines.push(format!(
            "{}: {}",
            diagnostic.severity.as_str(),
            diagnostic.message
        ));
    }

    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use dust_diagnostics::Diagnostic;
    use dust_driver::{CacheReport, CleanReport, CommandResult, DoctorReport, WatchReport};

    use super::*;

    #[test]
    fn help_lists_public_commands() {
        let help = render_help();

        assert!(help.contains("build"));
        assert!(help.contains("watch"));
        assert!(help.contains("clean"));
        assert!(help.contains("help"));
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
                diagnostics: vec![Diagnostic::warning("something happened")],
                ..CommandResult::default()
            },
        );

        assert!(rendered.contains("watch  scanned: 0  generated: 0  skipped: 0  time: 22ms"));
        assert!(rendered.contains("watch  cycles: 2  rebuilds: 1"));
        assert!(rendered.contains("warning: something happened"));
    }
}
