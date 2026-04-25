use std::path::PathBuf;

/// The batch phase currently being processed by the driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressPhase {
    /// A one-shot build batch.
    Build,
    /// The initial batch of a watch command.
    WatchInitial,
    /// A rebuild batch triggered during watch mode.
    WatchRebuild,
}

/// One progress event emitted by the Dust driver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressEvent {
    /// One new batch has started.
    StartedBatch {
        /// The phase being processed.
        phase: ProgressPhase,
        /// The number of source libraries in the batch.
        total: usize,
    },
    /// One source library has completed processing.
    FinishedLibrary {
        /// The phase being processed.
        phase: ProgressPhase,
        /// The number of completed libraries in the current batch.
        completed: usize,
        /// The total number of libraries in the current batch.
        total: usize,
        /// The completed source path.
        source_path: PathBuf,
        /// Whether the result came from the persistent cache.
        cached: bool,
        /// Whether the output file was written.
        written: bool,
        /// Whether the output differed from the previous file contents.
        changed: bool,
        /// Whether the library produced an error diagnostic.
        had_errors: bool,
        /// The elapsed processing time for this library in milliseconds.
        elapsed_ms: u128,
    },
}
