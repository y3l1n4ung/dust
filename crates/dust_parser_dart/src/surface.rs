use dust_text::TextRange;

use crate::{ParsedQueryCallSurface, ParsedTypeSurface};

/// A generation-relevant view of one parsed Dart file.
///
/// This type is intentionally smaller than a full AST. It carries only the
/// file surface that Dust needs for resolution and lowering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDartFileSurface {
    /// The full source span for the parsed file.
    pub span: TextRange,
    /// Top-level directives such as imports and parts.
    pub directives: Vec<ParsedDirective>,
    /// Top-level classes that were extracted from the source.
    pub classes: Vec<ParsedClassSurface>,
    /// Top-level enums that were extracted from the source.
    pub enums: Vec<ParsedEnumSurface>,
    /// Top-level mixins that were extracted from the source.
    pub mixins: Vec<ParsedMixinSurface>,
    /// Top-level extensions that were extracted from the source.
    pub extensions: Vec<ParsedExtensionSurface>,
    /// Top-level extension types that were extracted from the source.
    pub extension_types: Vec<ParsedExtensionTypeSurface>,
    /// Top-level functions that were extracted from the source.
    pub functions: Vec<ParsedFunctionSurface>,
    /// Top-level variables that were extracted from the source.
    pub variables: Vec<ParsedTopLevelVariableSurface>,
    /// Top-level typedefs that were extracted from the source.
    pub typedefs: Vec<ParsedTypedefSurface>,
    /// Dust DB query helper calls found in the source.
    pub query_calls: Vec<ParsedQueryCallSurface>,
}

impl ParsedDartFileSurface {
    /// Returns `true` if no generation-relevant surface facts were extracted.
    pub fn is_empty(&self) -> bool {
        self.directives.is_empty()
            && self.classes.is_empty()
            && self.enums.is_empty()
            && self.mixins.is_empty()
            && self.extensions.is_empty()
            && self.extension_types.is_empty()
            && self.functions.is_empty()
            && self.variables.is_empty()
            && self.typedefs.is_empty()
            && self.query_calls.is_empty()
    }
}

/// Compatibility alias while parser callers migrate to `ParsedDartFileSurface`.
pub type ParsedLibrarySurface = ParsedDartFileSurface;

/// One top-level Dart directive extracted from a library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedDirective {
    /// A `library` directive.
    Library {
        /// The declared dotted library name, if present.
        name: Option<String>,
        /// Metadata annotations attached to the library directive.
        annotations: Vec<ParsedAnnotation>,
        /// The source span of the full directive.
        span: TextRange,
    },
    /// An `import` directive.
    Import {
        /// The imported URI text without quotes.
        uri: String,
        /// The optional import prefix after `as`.
        prefix: Option<String>,
        /// Names included by `show` combinators.
        show: Vec<String>,
        /// Names excluded by `hide` combinators.
        hide: Vec<String>,
        /// Whether the import uses `deferred as`.
        is_deferred: bool,
        /// The source span of the full directive.
        span: TextRange,
    },
    /// An `export` directive.
    Export {
        /// The exported URI text without quotes.
        uri: String,
        /// The source span of the full directive.
        span: TextRange,
    },
    /// A `part` directive.
    Part {
        /// The part URI text without quotes.
        uri: String,
        /// The source span of the full directive.
        span: TextRange,
    },
    /// A `part of` directive.
    PartOf {
        /// The declared library name, if written as an identifier.
        library_name: Option<String>,
        /// The declared library URI, if written as a string.
        uri: Option<String>,
        /// The source span of the full directive.
        span: TextRange,
    },
}

impl ParsedDirective {
    /// Returns the source span of this directive.
    pub fn span(&self) -> TextRange {
        match self {
            Self::Library { span, .. }
            | Self::Import { span, .. }
            | Self::Export { span, .. }
            | Self::Part { span, .. }
            | Self::PartOf { span, .. } => *span,
        }
    }
}

/// A parsed class declaration relevant to Dust generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedClassKind {
    /// A normal Dart `class` declaration.
    Class,
    /// A Dart `sealed class` declaration.
    SealedClass,
    /// A Dart `mixin class` declaration.
    MixinClass,
}

/// A parsed class declaration relevant to Dust generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedClassSurface {
    /// The declaration kind.
    pub kind: ParsedClassKind,
    /// The class name.
    pub name: String,
    /// Whether the declaration is marked `abstract`.
    pub is_abstract: bool,
    /// Whether the declaration uses Dart's `interface class` form.
    pub is_interface: bool,
    /// The immediate superclass name, if the declaration has an `extends` clause.
    pub superclass_name: Option<String>,
    /// All metadata annotations attached to the class.
    pub annotations: Vec<ParsedAnnotation>,
    /// Extracted fields from the class body.
    pub fields: Vec<ParsedFieldSurface>,
    /// Extracted constructors from the class body.
    pub constructors: Vec<ParsedConstructorSurface>,
    /// Extracted methods from the class body.
    pub methods: Vec<ParsedMethodSurface>,
    /// The source span for the whole class declaration.
    pub span: TextRange,
}

/// One method extracted from a class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMethodSurface {
    /// The method name.
    pub name: String,
    /// Whether the method is marked `static`.
    pub is_static: bool,
    /// Whether the method is marked `external`.
    pub is_external: bool,
    /// All metadata annotations attached to the method.
    pub annotations: Vec<ParsedAnnotation>,
    /// The raw return type source, if present.
    pub return_type_source: Option<String>,
    /// Parsed return type facts, when provided by the parser backend.
    pub parsed_return_type: Option<ParsedTypeSurface>,
    /// Whether the method includes an implementation body.
    pub has_body: bool,
    /// The raw method body source, if available.
    pub body_source: Option<String>,
    /// The method parameters.
    pub params: Vec<ParsedMethodParamSurface>,
    /// The source span for the method declaration.
    pub span: TextRange,
}

/// One extracted method parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMethodParamSurface {
    /// The parameter name.
    pub name: String,
    /// All metadata annotations attached to the parameter.
    pub annotations: Vec<ParsedAnnotation>,
    /// The raw type source, if explicitly written.
    pub type_source: Option<String>,
    /// Parsed type facts, when provided by the parser backend.
    pub parsed_type: Option<ParsedTypeSurface>,
    /// The parameter kind.
    pub kind: ParameterKind,
    /// Whether the parameter has a default value.
    pub has_default: bool,
    /// The raw default value expression source, if explicitly written.
    pub default_value_source: Option<String>,
    /// The source span for the parameter.
    pub span: TextRange,
}

/// A parsed enum declaration relevant to Dust generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEnumSurface {
    /// The enum name
    pub name: String,
    /// All the metadata annotations attached to the enum.
    pub annotations: Vec<ParsedAnnotation>,
    /// Extracted variants from the enum body
    pub variants: Vec<ParsedEnumVariantSurface>,
    /// The source span for the whole enum declaration.
    pub span: TextRange,
}

/// One enum variant extracted from an enum declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEnumVariantSurface {
    /// The variant name.
    pub name: String,
    /// The metadata annotations attached to the variant.
    pub annotations: Vec<ParsedAnnotation>,
    /// The source span for the variant.
    pub span: TextRange,
}

/// One parsed mixin declaration relevant to Dust generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMixinSurface {
    /// The mixin name.
    pub name: String,
    /// Metadata annotations attached to the mixin.
    pub annotations: Vec<ParsedAnnotation>,
    /// Extracted fields from the mixin body.
    pub fields: Vec<ParsedFieldSurface>,
    /// The source span for the mixin declaration.
    pub span: TextRange,
}

/// One parsed extension declaration relevant to Dust generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedExtensionSurface {
    /// The optional extension name.
    pub name: Option<String>,
    /// The raw `on` type source, if present.
    pub on_type_source: Option<String>,
    /// Parsed `on` type facts, when provided by the parser backend.
    pub parsed_on_type: Option<ParsedTypeSurface>,
    /// Metadata annotations attached to the extension.
    pub annotations: Vec<ParsedAnnotation>,
    /// The source span for the extension declaration.
    pub span: TextRange,
}

/// One parsed extension type declaration relevant to Dust generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedExtensionTypeSurface {
    /// The extension type name.
    pub name: String,
    /// The representation field name.
    pub representation_name: String,
    /// The raw representation field type source, if present.
    pub representation_type_source: Option<String>,
    /// Parsed representation type facts, when provided by the parser backend.
    pub parsed_representation_type: Option<ParsedTypeSurface>,
    /// Metadata annotations attached to the extension type.
    pub annotations: Vec<ParsedAnnotation>,
    /// The source span for the extension type declaration.
    pub span: TextRange,
}

/// One parsed top-level function declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFunctionSurface {
    /// The function name.
    pub name: String,
    /// The raw return type source, if present.
    pub return_type_source: Option<String>,
    /// Parsed return type facts, when provided by the parser backend.
    pub parsed_return_type: Option<ParsedTypeSurface>,
    /// The function parameters.
    pub params: Vec<ParsedMethodParamSurface>,
    /// Metadata annotations attached to the function.
    pub annotations: Vec<ParsedAnnotation>,
    /// The source span for the function signature.
    pub span: TextRange,
}

/// One parsed top-level variable declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTopLevelVariableSurface {
    /// The variable name.
    pub name: String,
    /// The raw declared type source, if present.
    pub type_source: Option<String>,
    /// Parsed type facts, when provided by the parser backend.
    pub parsed_type: Option<ParsedTypeSurface>,
    /// The raw initializer expression source, if present.
    pub initializer_source: Option<String>,
    /// The initializer expression span, if present.
    pub initializer_span: Option<TextRange>,
    /// Metadata annotations attached to the variable declaration.
    pub annotations: Vec<ParsedAnnotation>,
    /// The source span for the variable.
    pub span: TextRange,
}

/// One parsed typedef declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTypedefSurface {
    /// The typedef name.
    pub name: String,
    /// The raw aliased type or function signature source, if present.
    pub aliased_type_source: Option<String>,
    /// Parsed aliased type facts, when provided by the parser backend.
    pub parsed_aliased_type: Option<ParsedTypeSurface>,
    /// Metadata annotations attached to the typedef.
    pub annotations: Vec<ParsedAnnotation>,
    /// The source span for the typedef declaration.
    pub span: TextRange,
}

impl ParsedClassSurface {
    /// Returns `true` if the class has at least one annotation with this name.
    pub fn has_annotation(&self, annotation_name: &str) -> bool {
        self.annotations
            .iter()
            .any(|annotation| annotation.is_named(annotation_name))
    }

    /// Returns `true` when this declaration is a Dart `mixin class`.
    pub fn is_mixin_class(&self) -> bool {
        matches!(self.kind, ParsedClassKind::MixinClass)
    }

    /// Returns `true` when this declaration uses Dart's `interface class` form.
    pub fn is_interface_class(&self) -> bool {
        self.is_interface
    }
}

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
    /// A list literal.
    List,
    /// A set literal.
    Set,
    /// A map literal.
    Map,
    /// A record literal.
    Record,
    /// A constructor invocation.
    Constructor {
        /// Constructor/type source.
        name: String,
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

/// One field declaration extracted from a class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFieldSurface {
    /// The field name.
    pub name: String,
    /// All metadata annotations attached to the field declaration.
    pub annotations: Vec<ParsedAnnotation>,
    /// The raw type source, if the declaration had one.
    pub type_source: Option<String>,
    /// Parsed type facts, when provided by the parser backend.
    pub parsed_type: Option<ParsedTypeSurface>,
    /// Whether the field declaration contains an initializer.
    pub has_default: bool,
    /// The source span for the field.
    pub span: TextRange,
}

impl ParsedFieldSurface {
    /// Returns `true` if the field has at least one annotation with this name.
    pub fn has_annotation(&self, annotation_name: &str) -> bool {
        self.annotations
            .iter()
            .any(|annotation| annotation.is_named(annotation_name))
    }
}

/// The parameter style used by a constructor parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterKind {
    /// A positional parameter.
    Positional,
    /// A named parameter.
    Named,
}

/// One constructor extracted from a class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedConstructorSurface {
    /// The named constructor suffix, if present.
    pub name: Option<String>,
    /// Whether the constructor is declared with the `factory` modifier.
    pub is_factory: bool,
    /// All metadata annotations attached to the constructor.
    pub annotations: Vec<ParsedAnnotation>,
    /// The redirected target symbol reference, if the constructor redirects.
    pub redirected_target_source: Option<String>,
    /// The redirected target base name, if it could be extracted.
    pub redirected_target_name: Option<String>,
    /// The constructor parameters.
    pub params: Vec<ParsedConstructorParamSurface>,
    /// The source span for the constructor declaration.
    pub span: TextRange,
}

/// One extracted constructor parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedConstructorParamSurface {
    /// The parameter name.
    pub name: String,
    /// All metadata annotations attached to the parameter.
    pub annotations: Vec<ParsedAnnotation>,
    /// The raw type source, if explicitly written.
    pub type_source: Option<String>,
    /// Parsed type facts, when provided by the parser backend.
    pub parsed_type: Option<ParsedTypeSurface>,
    /// The parameter kind.
    pub kind: ParameterKind,
    /// Whether the parameter has a default value.
    pub has_default: bool,
    /// The raw default value expression source, if explicitly written.
    pub default_value_source: Option<String>,
    /// The source span for the parameter.
    pub span: TextRange,
}
