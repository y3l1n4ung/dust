#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Resolution helpers that map parsed Dust-relevant syntax into symbol-aware semantic data."]

mod catalog;
mod resolve;
mod resolve_support;
mod result;

pub use catalog::{ResolvedSymbol, SymbolCatalog, SymbolKind};
pub use resolve::{resolve_library, validate_generated_part_uri};
pub use result::{
    ResolveResult, ResolvedClass, ResolvedEnum, ResolvedEnumVariant, ResolvedField, ResolvedLibrary,
};
