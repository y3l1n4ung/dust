use crate::SpanIr;

/// A stable semantic identifier for a trait or config symbol.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolId(pub String);

impl SymbolId {
    /// Creates a symbol identifier from a fully qualified name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

/// One resolved trait application on a class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitApplicationIr {
    /// The resolved trait symbol.
    pub symbol: SymbolId,
    /// The source span of the trait annotation.
    pub span: SpanIr,
}

/// One resolved configuration application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigApplicationIr {
    /// The resolved config symbol.
    pub symbol: SymbolId,
    /// The raw annotation argument source, if the config was written with
    /// parentheses and arguments.
    pub arguments_source: Option<String>,
    /// The source span of the config annotation.
    pub span: SpanIr,
}
