//! Surface types for fields, constructors and their parameters.

use dust_text::TextRange;

use crate::{ParsedTypeSurface, surface::ParsedAnnotation};

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
