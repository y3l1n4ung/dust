use dust_diagnostics::Diagnostic;
use dust_ir::{ClassKindIr, ConfigApplicationIr, SpanIr, TraitApplicationIr};
use dust_parser_dart::{
    ParsedConstructorSurface, ParsedDirective, ParsedExtensionSurface, ParsedExtensionTypeSurface,
    ParsedFunctionSurface, ParsedMethodParamSurface, ParsedMethodSurface, ParsedMixinSurface,
    ParsedQueryCallSurface, ParsedTopLevelVariableSurface, ParsedTypeSurface, ParsedTypedefSurface,
};

/// One resolved field plus its field-level Dust configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedField {
    /// The field name.
    pub name: String,
    /// The raw type source, if one was declared.
    pub type_source: Option<String>,
    /// Parsed type facts, when provided by the parser backend.
    pub parsed_type: Option<ParsedTypeSurface>,
    /// Whether the field has a default initializer in the declaration.
    pub has_default: bool,
    /// The field source span.
    pub span: SpanIr,
    /// Resolved config applications attached to the field.
    pub configs: Vec<ConfigApplicationIr>,
}

/// One resolved method plus its method-level Dust configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedMethod {
    /// The parsed method surface.
    pub surface: ParsedMethodSurface,
    /// The method source span.
    pub span: SpanIr,
    /// Resolved trait applications (if any).
    pub traits: Vec<TraitApplicationIr>,
    /// Resolved config applications.
    pub configs: Vec<ConfigApplicationIr>,
    /// Resolved method parameters in declaration order.
    pub params: Vec<ResolvedMethodParam>,
}

/// One resolved method parameter plus its parameter-level Dust configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedMethodParam {
    /// The parsed parameter surface.
    pub surface: ParsedMethodParamSurface,
    /// The parameter source span.
    pub span: SpanIr,
    /// Resolved trait applications (if any).
    pub traits: Vec<TraitApplicationIr>,
    /// Resolved config applications.
    pub configs: Vec<ConfigApplicationIr>,
}

/// One resolved constructor plus its constructor-level Dust configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConstructor {
    /// The parsed constructor surface.
    pub surface: ParsedConstructorSurface,
    /// Resolved config applications.
    pub configs: Vec<ConfigApplicationIr>,
}

/// One resolved enum plus its resolved Dust symbols.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEnum {
    /// The enum name.
    pub name: String,
    /// The enum source span.
    pub span: SpanIr,
    /// The resolved variants.
    pub variants: Vec<ResolvedEnumVariant>,
    /// Resolved traits applications.
    pub traits: Vec<TraitApplicationIr>,
    /// Resolved config applications.
    pub configs: Vec<ConfigApplicationIr>,
}
/// One resolved enum variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEnumVariant {
    /// The variant name.
    pub name: String,
    /// The variant source span.
    pub span: SpanIr,
    /// Resolved config applications.
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
    /// Whether the declaration uses Dart's `interface class` form.
    pub is_interface: bool,
    /// The immediate superclass name, if one was declared.
    pub superclass_name: Option<String>,
    /// The class source span.
    pub span: SpanIr,
    /// The resolved fields preserved for later lowering.
    pub fields: Vec<ResolvedField>,
    /// The parsed constructors preserved for later lowering.
    pub constructors: Vec<ResolvedConstructor>,
    /// The resolved methods preserved for later lowering.
    pub methods: Vec<ResolvedMethod>,
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
    ///  The resolved enums in declaration order.
    pub enums: Vec<ResolvedEnum>,
    /// Parsed mixins preserved for semantic lowering.
    pub mixins: Vec<ParsedMixinSurface>,
    /// Parsed extensions preserved for semantic lowering.
    pub extensions: Vec<ParsedExtensionSurface>,
    /// Parsed extension types preserved for semantic lowering.
    pub extension_types: Vec<ParsedExtensionTypeSurface>,
    /// Parsed top-level functions preserved for semantic lowering.
    pub functions: Vec<ParsedFunctionSurface>,
    /// Parsed top-level variables preserved for semantic lowering.
    pub variables: Vec<ParsedTopLevelVariableSurface>,
    /// Parsed typedefs preserved for semantic lowering.
    pub typedefs: Vec<ParsedTypedefSurface>,
    /// Parsed query helper calls preserved for semantic lowering.
    pub query_calls: Vec<ParsedQueryCallSurface>,
}

/// The result of resolving one parsed library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveResult {
    /// The resolved library state.
    pub library: ResolvedLibrary,
    /// Diagnostics emitted during resolution.
    pub diagnostics: Vec<Diagnostic>,
}
