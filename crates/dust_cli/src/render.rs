use dust_driver::CommandResult;

use crate::args::CliCommand;

const BANNER: &str = include_str!("../../../../brand/dust-logo-cli.txt");
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
        "  --root <path>       Use a specific workspace root",
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
