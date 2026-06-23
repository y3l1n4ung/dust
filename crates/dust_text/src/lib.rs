#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Low-level text primitives for Dust."]

/// Source-file identifiers.
mod file_id;
/// Line and column indexing.
mod line_index;
/// Byte offsets and ranges.
mod range;
/// Owned source text storage.
mod source_text;

pub use file_id::FileId;
pub use line_index::{LineCol, LineIndex};
pub use range::{TextRange, TextSize};
pub use source_text::SourceText;
