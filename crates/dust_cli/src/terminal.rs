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
