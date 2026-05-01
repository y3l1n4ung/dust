use crate::{
    build::{
        batch::ProgressReporter,
        process::{IndexedBuildOutcome, PendingLibrary, ProcessingConfig, process_pending_library},
        work::round_robin_groups,
    },
};

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

pub(super) fn process_pending_parallel(
    pending: Vec<PendingLibrary>,
    jobs: usize,
    processing: &ProcessingConfig<'_>,
    reporter: &ProgressReporter<'_>,
) -> Vec<IndexedBuildOutcome> {
    let groups = round_robin_groups(pending, jobs);

    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(groups.len());
        for group in groups {
            handles.push(scope.spawn(move || {
                group
                    .into_iter()
                    .map(|pending| process_pending_library(pending, processing, reporter))
                    .collect::<Vec<_>>()
            }));
        }

        let mut processed = Vec::new();
        for handle in handles {
            processed.extend(handle.join().expect("worker thread must not panic"));
        }
        processed
    })
}
