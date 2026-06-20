#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Resolution helpers that map parsed Dust-relevant syntax into symbol-aware semantic data."]

/// Annotation lowering helpers.
mod annotations;
/// Symbol catalog lookup.
mod catalog;
/// Library resolution entry points.
mod resolve;
/// Shared declaration resolution helpers.
mod resolve_support;
/// Resolved library result types.
mod result;

pub use annotations::annotation_ir_from_parsed;
pub use catalog::{ResolvedSymbol, SymbolCatalog, SymbolKind};
pub use resolve::{
    resolve_library, resolve_library_with_partless_configs, validate_generated_part_uri,
};
pub use result::{
    ResolveResult, ResolvedClass, ResolvedEnum, ResolvedEnumVariant, ResolvedField,
    ResolvedLibrary, ResolvedMethod, ResolvedMethodParam,
};
