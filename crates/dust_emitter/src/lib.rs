#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Deterministic `.g.dart` assembly and file writing for Dust."]

mod emit;
mod format;
mod merge;
mod write;
mod writer;

pub use emit::{EmitResult, emit_library};
pub use write::{WriteResult, write_library};
