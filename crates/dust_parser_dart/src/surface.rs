use dust_text::TextRange;

use crate::ParsedQueryCallSurface;

/// Surface types for annotations and their argument values.
mod annotations;
/// Surface types for top-level and member declarations.
mod declarations;
/// Surface types for fields, constructors and parameters.
mod members;

pub use self::{annotations::*, declarations::*, members::*};

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
    /// Database query helper calls found in the source.
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
