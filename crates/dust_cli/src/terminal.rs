use std::{
    io::{IsTerminal, Write},
    path::Path,
    sync::{Arc, Mutex},
};

use dust_driver::{ProgressEvent, ProgressPhase};

use crate::args::CliCommand;

pub(crate) type ProgressHandle = Arc<Mutex<TerminalProgress>>;

pub(crate) fn create_progress_handle(command: &CliCommand) -> Option<ProgressHandle> {
    if !std::io::stdout().is_terminal() {
        return None;
    }

    if !matches!(command, CliCommand::Build | CliCommand::Watch) {
        return None;
    }

    Some(Arc::new(Mutex::new(TerminalProgress::default())))
}

pub(crate) fn handle_progress(handle: &ProgressHandle, event: ProgressEvent) {
    let mut progress = handle
        .lock()
        .expect("terminal progress lock must be available");
    progress.handle(event);
}

pub(crate) fn finish_progress(handle: &ProgressHandle) {
    let mut progress = handle
        .lock()
        .expect("terminal progress lock must be available");
    progress.finish();
}

#[derive(Default)]
pub(crate) struct TerminalProgress {
    last_len: usize,
    active: bool,
}

impl TerminalProgress {
    fn handle(&mut self, event: ProgressEvent) {
        match event {
            ProgressEvent::StartedBatch { phase, total } => {
                self.active = true;
                self.render_line(&format!(
                    "{} {}",
                    progress_label(phase),
                    render_bar(0, total)
                ));
            }
            ProgressEvent::FinishedLibrary {
                phase,
                completed,
                total,
                source_path,
                written,
                had_errors,
                elapsed_ms,
                ..
            } => {
                let status = if had_errors {
                    "err"
                } else if written {
                    "gen"
                } else {
                    "skip"
                };
                self.render_line(&format!(
                    "{} {} {status} {} {}ms",
                    progress_label(phase),
                    render_bar(completed, total),
                    display_name(&source_path),
                    elapsed_ms,
                ));
            }
        }
    }

    fn finish(&mut self) {
        if !self.active {
            return;
        }

        let mut stderr = std::io::stderr().lock();
        let clear = " ".repeat(self.last_len);
        let _ = write!(stderr, "\r{clear}\r");
        let _ = stderr.flush();
        self.last_len = 0;
        self.active = false;
    }

    fn render_line(&mut self, line: &str) {
        let mut stderr = std::io::stderr().lock();
        let padding = self.last_len.saturating_sub(line.len());
        let clear = " ".repeat(padding);
        let _ = write!(stderr, "\r{line}{clear}");
        let _ = stderr.flush();
        self.last_len = line.len();
    }
}

fn render_bar(completed: usize, total: usize) -> String {
    let total = total.max(1);
    let width = 24;
    let filled = completed.saturating_mul(width) / total;
    let empty = width.saturating_sub(filled);
    format!(
        "[{}{}] {completed}/{total}",
        "#".repeat(filled),
        "-".repeat(empty),
    )
}

fn progress_label(phase: ProgressPhase) -> &'static str {
    match phase {
        ProgressPhase::Build => "build",
        ProgressPhase::WatchInitial => "watch:init",
        ProgressPhase::WatchRebuild => "watch:rebuild",
    }
}

fn display_name(path: &Path) -> String {
    if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
        name.to_owned()
    } else {
        path.to_string_lossy().into_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use dust_driver::{ProgressEvent, ProgressPhase};

    use super::*;

    #[test]
    fn render_bar_handles_progress_and_empty_batches() {
        assert_eq!(render_bar(0, 0), "[------------------------] 0/1");
        assert_eq!(render_bar(6, 12), "[############------------] 6/12");
    }

    #[test]
    fn progress_labels_match_cli_output_contract() {
        assert_eq!(progress_label(ProgressPhase::Build), "build");
        assert_eq!(progress_label(ProgressPhase::WatchInitial), "watch:init");
        assert_eq!(progress_label(ProgressPhase::WatchRebuild), "watch:rebuild");
    }

    #[test]
    fn display_name_prefers_file_name() {
        assert_eq!(
            display_name(Path::new("/tmp/example/user.dart")),
            "user.dart".to_owned()
        );

        let fallback = PathBuf::from("/");
        assert_eq!(display_name(&fallback), "/".to_owned());
    }

    #[test]
    fn terminal_progress_tracks_activity_and_resets_on_finish() {
        let mut progress = TerminalProgress::default();
        progress.handle(ProgressEvent::StartedBatch {
            phase: ProgressPhase::Build,
            total: 3,
        });
        assert!(progress.active);
        assert!(progress.last_len > 0);

        progress.handle(ProgressEvent::FinishedLibrary {
            phase: ProgressPhase::Build,
            completed: 1,
            total: 3,
            source_path: PathBuf::from("/tmp/lib/user.dart"),
            cached: false,
            written: true,
            changed: true,
            had_errors: false,
            elapsed_ms: 12,
        });
        assert!(progress.last_len > 0);

        progress.finish();
        assert!(!progress.active);
        assert_eq!(progress.last_len, 0);
    }
}
