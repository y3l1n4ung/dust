use dust_text::SourceText;

use crate::{ParseOptions, ParseResult};

/// A parser backend capable of turning one source file into a Dust parse result.
///
/// This trait allows Dust to keep the public parser contract stable while
/// swapping backend implementations, such as tree-sitter, behind the same API.
pub trait ParseBackend: Send + Sync {
    /// Parses one source file using the provided options.
    fn parse_file(&self, source: &SourceText, options: ParseOptions) -> ParseResult;
}

/// Runs parsing through an injected backend.
pub fn parse_file_with_backend(
    backend: &dyn ParseBackend,
    source: &SourceText,
    options: ParseOptions,
) -> ParseResult {
    backend.parse_file(source, options)
}
