use std::{
    io::{IsTerminal, Write},
    path::Path,
    sync::{Arc, Mutex},
};

use dust_driver::{ProgressEvent, ProgressPhase};

use crate::args::CliCommand;

/// Shared terminal progress state guarded for driver callbacks.
pub(crate) type ProgressHandle = Arc<Mutex<TerminalProgress>>;

/// Creates a progress handle when stdout is interactive and command supports progress.
pub(crate) fn create_progress_handle(
    command: &CliCommand,
    ai_mode: bool,
) -> Option<ProgressHandle> {
    if ai_mode {
        return None;
    }

    if !std::io::stdout().is_terminal() {
        return None;
    }

    if !matches!(command, CliCommand::Build | CliCommand::Watch) {
        return None;
    }

    Some(Arc::new(Mutex::new(TerminalProgress::default())))
}

/// Applies one driver progress event to terminal state.
pub(crate) fn handle_progress(handle: &ProgressHandle, event: ProgressEvent) {
    let mut progress = handle
        .lock()
        .expect("terminal progress lock must be available");
    progress.handle(event);
}

/// Clears any active terminal progress line.
pub(crate) fn finish_progress(handle: &ProgressHandle) {
    let mut progress = handle
        .lock()
        .expect("terminal progress lock must be available");
    progress.finish();
}

/// Mutable state for one-line terminal progress rendering.
#[derive(Default)]
pub(crate) struct TerminalProgress {
    /// Length of the last rendered line so it can be cleared.
    last_len: usize,
    /// Whether a progress line is currently active.
    active: bool,
}

impl TerminalProgress {
    /// Handles one progress event and renders the corresponding line.
    fn handle(&mut self, event: ProgressEvent) {
        self.active = true;
        self.render_line(&render_progress_event(&event));
    }

    /// Clears the active progress line from stderr.
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

    /// Renders one carriage-return progress line to stderr.
    fn render_line(&mut self, line: &str) {
        let mut stderr = std::io::stderr().lock();
        let padding = self.last_len.saturating_sub(line.len());
        let clear = " ".repeat(padding);
        let _ = write!(stderr, "\r{line}{clear}");
        let _ = stderr.flush();
        self.last_len = line.len();
    }
}

/// Renders one progress event as a terminal status line.
fn render_progress_event(event: &ProgressEvent) -> String {
    match event {
        ProgressEvent::StartedBatch { phase, total } => {
            format!("{} {}", progress_label(*phase), render_bar(0, *total))
        }
        ProgressEvent::FinishedLibrary {
            phase,
            completed,
            total,
            source_path,
            cached,
            routed,
            written,
            had_errors,
            elapsed_ms,
            ..
        } => format!(
            "{} {} {} {} {}ms",
            progress_label(*phase),
            render_bar(*completed, *total),
            progress_status(*had_errors, *written, *routed, *cached),
            display_name(source_path),
            elapsed_ms,
        ),
    }
}

/// Chooses the compact status label for one finished library.
fn progress_status(had_errors: bool, written: bool, routed: bool, cached: bool) -> &'static str {
    if had_errors {
        "err"
    } else if written {
        "gen"
    } else if routed {
        "route"
    } else if cached {
        "cache"
    } else {
        "skip"
    }
}

/// Renders a fixed-width ASCII progress bar.
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

/// Returns the terminal label for a progress phase.
fn progress_label(phase: ProgressPhase) -> &'static str {
    match phase {
        ProgressPhase::Build => "build",
        ProgressPhase::WatchInitial => "watch:init",
        ProgressPhase::WatchRebuild => "watch:rebuild",
    }
}

/// Returns a compact display name for a source path.
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
    fn progress_status_prefers_errors_over_write_state() {
        assert_eq!(progress_status(true, true, true, true), "err");
        assert_eq!(progress_status(false, true, true, true), "gen");
        assert_eq!(progress_status(false, false, true, true), "route");
        assert_eq!(progress_status(false, false, false, true), "cache");
        assert_eq!(progress_status(false, false, false, false), "skip");
    }

    #[test]
    fn render_progress_event_formats_started_and_finished_lines() {
        assert_eq!(
            render_progress_event(&ProgressEvent::StartedBatch {
                phase: ProgressPhase::Build,
                total: 3,
            }),
            "build [------------------------] 0/3"
        );
        assert_eq!(
            render_progress_event(&ProgressEvent::FinishedLibrary {
                phase: ProgressPhase::WatchRebuild,
                completed: 2,
                total: 4,
                source_path: PathBuf::from("/tmp/lib/user.dart"),
                cached: false,
                routed: false,
                written: true,
                changed: true,
                had_errors: false,
                elapsed_ms: 9,
            }),
            "watch:rebuild [############------------] 2/4 gen user.dart 9ms"
        );
        assert_eq!(
            render_progress_event(&ProgressEvent::FinishedLibrary {
                phase: ProgressPhase::Build,
                completed: 3,
                total: 4,
                source_path: PathBuf::from("/tmp/lib/dashboard_page.dart"),
                cached: false,
                routed: true,
                written: false,
                changed: false,
                had_errors: false,
                elapsed_ms: 0,
            }),
            "build [##################------] 3/4 route dashboard_page.dart 0ms"
        );
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
            routed: false,
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

    #[test]
    fn ai_mode_disables_progress_handle() {
        assert!(create_progress_handle(&CliCommand::Build, true).is_none());
    }
}
