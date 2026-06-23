#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Persistent workspace cache storage for Dust builds."]

/// JSON-backed workspace build cache implementation.
mod store;

pub use store::{CACHE_SCHEMA_VERSION, CacheEntry, WorkspaceCache};
