use crate::build::{
    batch::ProgressReporter,
    process::{IndexedBuildOutcome, PendingLibrary, ProcessingConfig, process_pending_library},
    work::round_robin_groups,
};

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Processes pending libraries on the current thread in discovery order.
pub(super) fn process_pending_serial(
    pending: Vec<PendingLibrary>,
    fail_fast: bool,
    processing: &ProcessingConfig<'_>,
    reporter: &ProgressReporter<'_>,
) -> Vec<IndexedBuildOutcome> {
    let mut processed = Vec::with_capacity(pending.len());

    for pending in pending {
        let outcome = process_pending_library(pending, processing, reporter);
        let has_error = outcome
            .outcome
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error());
        processed.push(outcome);

        if fail_fast && has_error {
            break;
        }
    }

    processed
}

/// Processes pending libraries across scoped worker threads.
pub(super) fn process_pending_parallel(
    pending: Vec<PendingLibrary>,
    jobs: usize,
    fail_fast: bool,
    processing: &ProcessingConfig<'_>,
    reporter: &ProgressReporter<'_>,
) -> Vec<IndexedBuildOutcome> {
    let groups = round_robin_groups(pending, jobs);
    let stop = Arc::new(AtomicBool::new(false));

    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(groups.len());
        for group in groups {
            let stop = Arc::clone(&stop);
            handles.push(scope.spawn(move || {
                let mut processed = Vec::with_capacity(group.len());
                for pending in group {
                    if fail_fast && stop.load(Ordering::Relaxed) {
                        break;
                    }
                    let outcome = process_pending_library(pending, processing, reporter);
                    if fail_fast
                        && outcome
                            .outcome
                            .diagnostics
                            .iter()
                            .any(|diagnostic| diagnostic.is_error())
                    {
                        stop.store(true, Ordering::Relaxed);
                    }
                    processed.push(outcome);
                }
                processed
            }));
        }

        let mut processed = Vec::new();
        for handle in handles {
            processed.extend(handle.join().expect("worker thread must not panic"));
        }
        processed
    })
}
