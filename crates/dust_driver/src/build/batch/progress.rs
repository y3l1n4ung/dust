use std::sync::atomic::{AtomicUsize, Ordering};

use dust_workspace::SourceLibrary;

use crate::progress::{ProgressEvent, ProgressPhase};

use super::ProgressCallback;

/// Emits progress events for batch processing.
#[derive(Clone, Copy)]
pub(crate) struct ProgressReporter<'a> {
    /// Optional callback supplied by the caller.
    progress: Option<&'a ProgressCallback<'a>>,
    /// Shared completed-library counter.
    completed: &'a AtomicUsize,
    /// Command phase reported with each event.
    phase: ProgressPhase,
    /// Number of libraries in the active batch.
    total: usize,
}

/// Per-library data used to finish one progress event.
pub(crate) struct ProgressSnapshot<'a> {
    /// Library that finished processing.
    pub(crate) library: &'a SourceLibrary,
    /// Whether the library was reused from cache.
    pub(crate) cached: bool,
    /// Whether the library was handled by route-only generation.
    pub(crate) routed: bool,
    /// Whether generated output was written.
    pub(crate) written: bool,
    /// Whether generated output differed from previous output.
    pub(crate) changed: bool,
    /// Whether processing produced error diagnostics.
    pub(crate) had_errors: bool,
    /// Elapsed processing time in milliseconds.
    pub(crate) elapsed_ms: u128,
}

impl<'a> ProgressReporter<'a> {
    /// Creates a reporter bound to one batch and completed counter.
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

    /// Emits the batch-start event when a callback exists.
    pub(crate) fn started_batch(&self) {
        if let Some(progress) = self.progress {
            progress(ProgressEvent::StartedBatch {
                phase: self.phase,
                total: self.total,
            });
        }
    }

    /// Emits the completed-library event and advances the completed counter.
    pub(crate) fn finish(&self, snapshot: ProgressSnapshot<'_>) {
        if let Some(progress) = self.progress {
            let completed = self.completed.fetch_add(1, Ordering::SeqCst) + 1;
            progress(ProgressEvent::FinishedLibrary {
                phase: self.phase,
                completed,
                total: self.total,
                source_path: snapshot.library.source_path.clone(),
                cached: snapshot.cached,
                routed: snapshot.routed,
                written: snapshot.written,
                changed: snapshot.changed,
                had_errors: snapshot.had_errors,
                elapsed_ms: snapshot.elapsed_ms,
            });
        }
    }
}
