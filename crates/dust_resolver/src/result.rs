use dust_diagnostics::Diagnostic;
use dust_ir::{ClassKindIr, ConfigApplicationIr, SpanIr, TraitApplicationIr};
use dust_parser_dart::{ParsedConstructorSurface, ParsedDirective};

/// One resolved field plus its field-level Dust configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedField {
    /// The field name.
    pub name: String,
    /// The raw type source, if one was declared.
    pub type_source: Option<String>,
    /// Whether the field has a default initializer in the declaration.
    pub has_default: bool,
    /// The field source span.
    pub span: SpanIr,
    /// Resolved config applications attached to the field.
    pub configs: Vec<ConfigApplicationIr>,
}

/// One resolved class plus its resolved Dust symbols.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedClass {
    /// The declaration kind.
    pub kind: ClassKindIr,
    /// The class name.
    pub name: String,
    /// Whether the declaration is marked `abstract`.
    pub is_abstract: bool,
    /// The immediate superclass name, if one was declared.
    pub superclass_name: Option<String>,
    /// The class source span.
    pub span: SpanIr,
    /// The resolved fields preserved for later lowering.
    pub fields: Vec<ResolvedField>,
    /// The parsed constructors preserved for later lowering.
    pub constructors: Vec<ParsedConstructorSurface>,
    /// Resolved trait applications.
    pub traits: Vec<TraitApplicationIr>,
    /// Resolved config applications.
    pub configs: Vec<ConfigApplicationIr>,
}

/// One resolved library ready for lowering into semantic IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLibrary {
    /// The original source path.
    pub source_path: String,
    /// The generated output path derived from the source path.
    pub output_path: String,
    /// The library source span.
    pub span: SpanIr,
    /// The parsed directives preserved after resolution.
    pub directives: Vec<ParsedDirective>,
    /// The declared generated part URI if present.
    pub part_uri: Option<String>,
    /// The resolved classes in declaration order.
    pub classes: Vec<ResolvedClass>,
}

/// The result of resolving one parsed library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveResult {
    /// The resolved library state.
    pub library: ResolvedLibrary,
    /// Diagnostics emitted during resolution.
    pub diagnostics: Vec<Diagnostic>,
}
