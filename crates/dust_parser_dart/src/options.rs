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
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            source_kind: SourceKind::Library,
        }
    }
}
