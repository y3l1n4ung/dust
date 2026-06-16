#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Backend-neutral parser contracts and extracted library surface types for Dust."]

mod annotation_args;
mod annotation_value;
mod backend;
mod options;
mod query_call;
mod result;
mod surface;
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
