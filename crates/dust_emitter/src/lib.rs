#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Deterministic `.g.dart` assembly and file writing for Dust."]

/// In-memory generated source assembly.
mod emit;
/// Generated source final formatting.
mod format;
/// Plugin contribution merging.
mod merge;
/// Filesystem write orchestration.
mod write;
/// Small Dart source writer.
mod writer;

pub use emit::{EmitResult, emit_library, emit_library_with_plan, hash_output_set};
pub use write::{
    AuxiliaryWriteResult, WriteResult, persist_emit_result, write_library, write_library_with_plan,
};
