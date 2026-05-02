#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Diagnostics and source labels for Dust."]

mod diagnostic;
mod label;
mod render;
mod severity;

pub use diagnostic::Diagnostic;
pub use label::SourceLabel;
pub use render::{DiagnosticFileContext, render_to_string, render_to_string_with_files};
pub use severity::Severity;
