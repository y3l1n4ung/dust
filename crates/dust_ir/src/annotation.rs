use std::collections::BTreeMap;

use crate::{SpanIr, SymbolId};

/// A Dart identifier or dotted/prefixed name with source spelling preserved.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NameIr {
    /// The source spelling for this name.
    pub source: String,
    /// The final segment used for short-name matching.
    pub short: String,
    /// The optional import prefix before the short name.
    pub prefix: Option<String>,
    /// The source span for this name.
    pub span: SpanIr,
}

/// A Dart expression preserved as source after parser boundary extraction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExprSourceIr {
    /// The expression source text.
    pub source: String,
    /// The source span for this expression.
    pub span: SpanIr,
}

/// A structured constant annotation value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationValueIr {
    /// The `null` literal.
    Null,
    /// A boolean literal.
    Bool(bool),
    /// A string literal with delimiters removed.
    String(String),
    /// A numeric literal kept as source for exact interpretation.
    Number {
        /// The numeric literal source.
        source: String,
        /// The numeric literal kind.
        kind: AnnotationNumberKindIr,
    },
    /// A list literal.
    List(Vec<AnnotationValueIr>),
    /// A set literal.
    Set(Vec<AnnotationValueIr>),
    /// A map literal.
    Map(Vec<(AnnotationValueIr, AnnotationValueIr)>),
    /// A named record literal.
    Record(Vec<(String, AnnotationValueIr)>),
    /// A constant constructor invocation.
    Constructor {
        /// The constructor/type name.
        name: NameIr,
        /// Positional constructor arguments.
        positional_args: Vec<AnnotationValueIr>,
        /// Named constructor arguments.
        named_args: BTreeMap<String, AnnotationValueIr>,
    },
    /// A member, type, or function reference.
    Member(NameIr),
    /// An expression shape Dust preserves but does not semantically parse yet.
    Expression(ExprSourceIr),
}

/// Semantic annotation numeric literal kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationNumberKindIr {
    /// An integer literal.
    Int,
    /// A floating point literal.
    Double,
}

/// One normalized Dart annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationIr {
    /// The full annotation name as written, without `@`.
    pub raw_name: String,
    /// The final annotation name segment.
    pub short_name: String,
    /// The optional import prefix before the annotation name.
    pub prefix: Option<String>,
    /// Positional annotation arguments.
    pub positional_args: Vec<AnnotationValueIr>,
    /// Named annotation arguments.
    pub named_args: BTreeMap<String, AnnotationValueIr>,
    /// The resolved symbol for this annotation, when resolution has run.
    pub resolved_symbol: Option<SymbolId>,
    /// The source span for this annotation.
    pub span: SpanIr,
}
