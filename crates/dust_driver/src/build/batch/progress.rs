use std::sync::atomic::{AtomicUsize, Ordering};

use dust_workspace::SourceLibrary;

use crate::progress::{ProgressEvent, ProgressPhase};

use super::ProgressCallback;

#[derive(Clone, Copy)]
pub(crate) struct ProgressReporter<'a> {
    progress: Option<&'a ProgressCallback<'a>>,
    completed: &'a AtomicUsize,
    phase: ProgressPhase,
    total: usize,
}

pub(crate) struct ProgressSnapshot<'a> {
    pub(crate) library: &'a SourceLibrary,
    pub(crate) cached: bool,
    pub(crate) written: bool,
    pub(crate) changed: bool,
    pub(crate) had_errors: bool,
    pub(crate) elapsed_ms: u128,
}

impl<'a> ProgressReporter<'a> {
    pub(crate) fn new(
        progress: Option<&'a ProgressCallback<'a>>,
        completed: &'a AtomicUsize,
        phase: ProgressPhase,
        total: usize,
    ) -> Self {
        Self {
            progress,
            completed,
            phase,
            total,
        }
    }

    pub(crate) fn started_batch(&self) {
        if let Some(progress) = self.progress {
            progress(ProgressEvent::StartedBatch {
                phase: self.phase,
                total: self.total,
            });
        }
    }

    pub(crate) fn finish(&self, snapshot: ProgressSnapshot<'_>) {
        if let Some(progress) = self.progress {
            let completed = self.completed.fetch_add(1, Ordering::SeqCst) + 1;
            progress(ProgressEvent::FinishedLibrary {
                phase: self.phase,
                completed,
                total: self.total,
                source_path: snapshot.library.source_path.clone(),
                cached: snapshot.cached,
                written: snapshot.written,
                changed: snapshot.changed,
                had_errors: snapshot.had_errors,
                elapsed_ms: snapshot.elapsed_ms,
            });
        }
    }
}
