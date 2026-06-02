#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Backend-neutral parser contracts and extracted library surface types for Dust."]

mod annotation_args;
mod backend;
mod options;
mod query_call;
mod result;
mod surface;

pub use backend::{ParseBackend, parse_file_with_backend};
pub use options::{ParseOptions, SourceKind};
pub use query_call::{ParsedQueryCallSurface, ParsedQueryFunction};
pub use result::ParseResult;
pub use surface::{
    ParameterKind, ParsedAnnotation, ParsedClassKind, ParsedClassSurface,
    ParsedConstructorParamSurface, ParsedConstructorSurface, ParsedDirective, ParsedEnumSurface,
    ParsedEnumVariantSurface, ParsedFieldSurface, ParsedLibrarySurface, ParsedMethodParamSurface,
    ParsedMethodSurface,
};
