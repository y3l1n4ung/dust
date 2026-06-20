use crate::{NameIr, SpanIr};

/// A Dart library declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryDeclIr {
    /// The declared library name, if present.
    pub name: Option<NameIr>,
    /// The source span for the declaration.
    pub span: SpanIr,
}

/// A Dart import directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportIr {
    /// The imported URI without quotes.
    pub uri: String,
    /// The optional import prefix.
    pub prefix: Option<String>,
    /// Names included by `show` combinators.
    pub show: Vec<String>,
    /// Names excluded by `hide` combinators.
    pub hide: Vec<String>,
    /// Whether the import uses `deferred as`.
    pub is_deferred: bool,
    /// The source span for the directive.
    pub span: SpanIr,
}

/// A Dart export directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportIr {
    /// The exported URI without quotes.
    pub uri: String,
    /// The source span for the directive.
    pub span: SpanIr,
}

/// A Dart part directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartIr {
    /// The part URI without quotes.
    pub uri: String,
    /// The source span for the directive.
    pub span: SpanIr,
}

/// A Dart part-of directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartOfIr {
    /// The declared library name, if written as an identifier.
    pub library_name: Option<NameIr>,
    /// The declared library URI, if written as a string.
    pub uri: Option<String>,
    /// The source span for the directive.
    pub span: SpanIr,
}
