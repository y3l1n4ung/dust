#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Diagnostics and source labels for Dust."]

/// Diagnostic builders.
mod diagnostic;
/// Source label metadata.
mod label;
/// Plain-text diagnostic rendering.
mod render;
/// Diagnostic severity levels.
mod severity;

pub use diagnostic::Diagnostic;
pub use label::SourceLabel;
pub use render::{DiagnosticFileContext, render_to_string, render_to_string_with_files};
pub use severity::Severity;
