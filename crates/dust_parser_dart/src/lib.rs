#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Backend-neutral parser contracts and extracted library surface types for Dust."]

/// Annotation argument helpers.
mod annotation_args;
/// Constant annotation value parsing.
mod annotation_value;
/// Parser backend abstraction.
mod backend;
/// Parser options.
mod options;
/// Query call surface parsing.
mod query_call;
/// Parser result containers.
mod result;
/// Dart declaration surface types.
mod surface;
/// Dart type surface parsing.
mod type_surface;

pub use annotation_value::{AnnotationValue, parse_annotation_named_values};
pub use backend::{ParseBackend, parse_file_with_backend};
pub use options::{ParseOptions, SourceKind};
pub use query_call::{ParsedQueryCallSurface, ParsedQueryFunction};
pub use result::ParseResult;
pub use surface::{
    ParameterKind, ParsedAnnotation, ParsedAnnotationArgument, ParsedAnnotationArguments,
    ParsedAnnotationNamedArgument, ParsedClassKind, ParsedClassSurface,
    ParsedConstructorParamSurface, ParsedConstructorSurface, ParsedDartFileSurface,
    ParsedDirective, ParsedEnumSurface, ParsedEnumVariantSurface, ParsedExtensionSurface,
    ParsedExtensionTypeSurface, ParsedFieldSurface, ParsedFunctionSurface, ParsedLibrarySurface,
    ParsedMethodParamSurface, ParsedMethodSurface, ParsedMixinSurface,
    ParsedTopLevelVariableSurface, ParsedTypedefSurface,
};
pub use type_surface::{ParsedTypeKind, ParsedTypeSurface};
