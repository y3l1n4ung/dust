//! Surface types for an annotation and the argument values inside it.

use dust_text::TextRange;

/// One metadata annotation attached to a declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAnnotation {
    /// The short annotation name without `@` or import prefix.
    pub name: String,
    /// The optional import prefix before the short name.
    pub prefix: Option<String>,
    /// The full annotation name without `@`.
    pub qualified_name: String,
    /// The raw argument source, if present.
    pub arguments_source: Option<String>,
    /// Parsed argument facts, when provided by the parser backend.
    pub parsed_arguments: Option<ParsedAnnotationArguments>,
    /// The source span for the full annotation.
    pub span: TextRange,
}

impl ParsedAnnotation {
    /// Returns `true` when the parsed short annotation name matches `annotation_name`.
    ///
    /// Import prefixes are intentionally ignored here. For example, both
    /// `@Derive()` and `@d.Derive()` match `Derive`.
    pub fn is_named(&self, annotation_name: &str) -> bool {
        self.name == annotation_name
    }
}

/// Parsed annotation argument facts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedAnnotationArguments {
    /// Positional annotation arguments.
    pub positional: Vec<ParsedAnnotationArgument>,
    /// Named annotation arguments.
    pub named: Vec<ParsedAnnotationNamedArgument>,
}

/// A parser-owned annotation value with exact source and root CST kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAnnotationValue {
    /// The exact value expression source.
    pub source: String,
    /// The source span for this value expression.
    pub span: TextRange,
    /// The root value kind reported by the parser backend.
    pub kind: ParsedAnnotationValueRootKind,
}

/// Parser-owned annotation value root kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedAnnotationValueRootKind {
    /// The `null` literal.
    Null,
    /// A boolean literal.
    Bool(bool),
    /// A string literal with delimiters removed.
    String(String),
    /// A numeric literal.
    Number(ParsedAnnotationNumberKind),
    /// A list literal and its directly parsed elements.
    List(Vec<ParsedAnnotationValue>),
    /// A set literal and its directly parsed elements.
    Set(Vec<ParsedAnnotationValue>),
    /// A map literal and its directly parsed key/value pairs.
    Map(Vec<(ParsedAnnotationValue, ParsedAnnotationValue)>),
    /// A named record literal and its directly parsed fields.
    Record(Vec<(String, ParsedAnnotationValue)>),
    /// A constructor invocation.
    Constructor {
        /// Constructor/type source.
        name: String,
        /// Structured constructor arguments.
        arguments: Box<ParsedAnnotationArguments>,
    },
    /// A member, type, or function reference.
    Member(String),
    /// Any expression shape Dust preserves but does not semantically parse yet.
    Expression,
}

/// Parser-owned annotation numeric literal kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedAnnotationNumberKind {
    /// An integer literal.
    Int,
    /// A floating point literal.
    Double,
}

/// One positional annotation argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAnnotationArgument {
    /// The raw argument expression source.
    pub source: String,
    /// The parser-owned typed value, when available.
    pub value: Option<ParsedAnnotationValue>,
    /// The source span for this argument expression.
    pub span: TextRange,
}

/// One named annotation argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAnnotationNamedArgument {
    /// The argument name before `:`.
    pub name: String,
    /// The full named argument source, including `name:`.
    pub source: String,
    /// The raw value expression source.
    pub value_source: String,
    /// The parser-owned typed value, when available.
    pub value: Option<ParsedAnnotationValue>,
    /// The source span for the full named argument.
    pub span: TextRange,
    /// The source span for the value expression.
    pub value_span: TextRange,
}
