/// Cache input loading and fingerprint comparison.
mod cache_input;
/// Plugin registry selection for full and DB-only generation modes.
mod registry;
/// Stable toolchain hash inputs used for cache invalidation.
mod tool_hash;

pub(crate) use cache_input::{
    CacheFingerprint, load_library_input, matches_cache_metadata, read_workspace_config_hash,
    route_only_analysis,
};
pub(crate) use registry::{RegistrySelection, default_registry, registry_for_selection};
pub(crate) use tool_hash::{CodegenToolHash, codegen_tool_hash_for_selection, hash_text};
