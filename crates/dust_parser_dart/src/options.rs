use dust_dart_syntax::DartLanguageVersion;

/// The source role being parsed.
///
/// Dust currently focuses on library parsing. The enum exists now so later
/// source modes can be added without breaking the public parser contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// Parse the source as a regular Dart library file.
    Library,
}

/// Controls how one parser backend should parse a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseOptions {
    /// The expected source role.
    pub source_kind: SourceKind,
    /// Workspace-level Dart language version used when the source file does
    /// not declare its own `// @dart = x.y` language version comment.
    pub language_version: DartLanguageVersion,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            source_kind: SourceKind::Library,
            language_version: DartLanguageVersion::latest_known(),
        }
    }
}

impl ParseOptions {
    /// Returns the language version that applies to one source file.
    pub fn effective_language_version_for_source(self, source: &str) -> DartLanguageVersion {
        DartLanguageVersion::from_source_language_comment(source).unwrap_or(self.language_version)
    }
}
