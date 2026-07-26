#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Resolution helpers that map parsed Dust-relevant syntax into symbol-aware semantic data."]

/// Annotation lowering helpers.
mod annotations;
/// Symbol catalog lookup.
mod catalog;
/// Resolver-owned database configuration normalization.
mod db;
/// Resolver-owned HTTP configuration normalization.
mod http;
/// Library resolution entry points.
mod resolve;
/// Shared declaration resolution helpers.
mod resolve_support;
/// Resolved library result types.
mod result;
/// Resolver-owned route normalization.
mod route;
/// Resolver-owned SerDe normalization.
mod serde;
/// Resolver-owned state configuration normalization.
mod state;
/// Resolver-owned type lowering.
mod types;

pub use annotations::{annotation_ir_from_parsed, resolve_annotation_ir};
pub use catalog::{ResolvedSymbol, SymbolCatalog, SymbolKind};
pub use resolve::{
    resolve_library, resolve_library_with_partless_configs, validate_generated_part_uri,
};
pub use result::{
    ResolveResult, ResolvedClass, ResolvedConstructor, ResolvedEnum, ResolvedEnumVariant,
    ResolvedField, ResolvedLibrary, ResolvedMethod, ResolvedMethodParam,
};
pub use types::lower_type_ir;
