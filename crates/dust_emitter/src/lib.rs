#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Deterministic `.g.dart` assembly and file writing for Dust."]

mod emit;
mod format;
mod merge;
mod write;
mod writer;

pub use emit::{EmitResult, emit_library, emit_library_with_plan, hash_output_set};
pub use write::{
    AuxiliaryWriteResult, WriteResult, persist_emit_result, write_library, write_library_with_plan,
};
