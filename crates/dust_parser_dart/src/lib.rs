#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Backend-neutral parser contracts and extracted library surface types for Dust."]

mod backend;
mod options;
mod result;
mod surface;

pub use backend::{ParseBackend, parse_file_with_backend};
pub use options::{ParseOptions, SourceKind};
pub use result::ParseResult;
pub use surface::{
    ParameterKind, ParsedAnnotation, ParsedClassKind, ParsedClassSurface,
    ParsedConstructorParamSurface, ParsedConstructorSurface, ParsedDirective, ParsedFieldSurface,
    ParsedLibrarySurface,
};
