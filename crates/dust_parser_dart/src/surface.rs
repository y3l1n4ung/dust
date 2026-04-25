use dust_text::TextRange;

/// A generation-relevant view of one parsed Dart library.
///
/// This type is intentionally smaller than a full AST. It carries only the
/// library surface that Dust needs for resolution and lowering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLibrarySurface {
    /// The full source span for the parsed library.
    pub span: TextRange,
    /// Top-level directives such as imports and parts.
    pub directives: Vec<ParsedDirective>,
    /// Top-level classes that were extracted from the source.
    pub classes: Vec<ParsedClassSurface>,
}

impl ParsedLibrarySurface {
    /// Returns `true` if no directives and no classes were extracted.
    pub fn is_empty(&self) -> bool {
        self.directives.is_empty() && self.classes.is_empty()
    }
}

/// One top-level Dart directive extracted from a library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedDirective {
    /// An `import` directive.
    Import {
        /// The imported URI text without quotes.
        uri: String,
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
            Self::Import { span, .. }
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
    /// The immediate superclass name, if the declaration has an `extends` clause.
    pub superclass_name: Option<String>,
    /// All metadata annotations attached to the class.
    pub annotations: Vec<ParsedAnnotation>,
    /// Extracted fields from the class body.
    pub fields: Vec<ParsedFieldSurface>,
    /// Extracted constructors from the class body.
    pub constructors: Vec<ParsedConstructorSurface>,
    /// The source span for the whole class declaration.
    pub span: TextRange,
}

impl ParsedClassSurface {
    /// Returns `true` if the class has at least one annotation with this name.
    pub fn has_annotation(&self, annotation_name: &str) -> bool {
        self.annotations
            .iter()
            .any(|annotation| annotation.name == annotation_name)
    }

    /// Returns `true` when this declaration is a Dart `mixin class`.
    pub fn is_mixin_class(&self) -> bool {
        matches!(self.kind, ParsedClassKind::MixinClass)
    }
}

/// One metadata annotation attached to a declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAnnotation {
    /// The annotation name without `@`.
    pub name: String,
    /// The raw argument source, if present.
    pub arguments_source: Option<String>,
    /// The source span for the full annotation.
    pub span: TextRange,
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
            .any(|annotation| annotation.name == annotation_name)
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
    /// The raw type source, if explicitly written.
    pub type_source: Option<String>,
    /// The parameter kind.
    pub kind: ParameterKind,
    /// Whether the parameter has a default value.
    pub has_default: bool,
    /// The source span for the parameter.
    pub span: TextRange,
}
