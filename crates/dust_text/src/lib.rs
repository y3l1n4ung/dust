#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Low-level text primitives for Dust."]

mod file_id;
mod line_index;
mod range;
mod source_text;

pub use file_id::FileId;
pub use line_index::{LineCol, LineIndex};
pub use range::{TextRange, TextSize};
pub use source_text::SourceText;
