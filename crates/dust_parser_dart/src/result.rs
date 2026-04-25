use dust_diagnostics::Diagnostic;

use crate::{ParseOptions, ParsedLibrarySurface};

/// The backend-neutral result of parsing one Dart source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    /// The extracted library surface.
    pub library: ParsedLibrarySurface,
    /// Diagnostics emitted during parsing.
    pub diagnostics: Vec<Diagnostic>,
    /// The options used for this parse.
    pub options: ParseOptions,
}

impl ParseResult {
    /// Returns `true` if at least one diagnostic is present.
    pub fn has_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}
