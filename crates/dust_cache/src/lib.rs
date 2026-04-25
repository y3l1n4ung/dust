#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Persistent workspace cache storage for Dust builds."]

mod store;

pub use store::{CACHE_SCHEMA_VERSION, CacheEntry, WorkspaceCache};
