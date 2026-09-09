//! Surface types for declarations a library can hold: methods, enums,
//! mixins, extensions, functions, variables and typedefs.

use dust_text::TextRange;

use crate::{
    ParsedTypeSurface,
    surface::{
        ParameterKind, ParsedAnnotation, ParsedClassKind, ParsedClassSurface, ParsedFieldSurface,
    },
};

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
    /// Whether the parameter uses Dart's explicit `required` modifier.
    pub is_required: bool,
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
