use super::registry::RegistrySelection;

/// Stable hash of the active code generation logic and plugin set.
#[derive(Clone, Copy)]
pub(crate) struct CodegenToolHash {
    /// FNV-1a hash value used in cache entries.
    hash: u64,
}

impl CodegenToolHash {
    /// Returns the raw hash value persisted in cache metadata.
    pub(crate) fn value(self) -> u64 {
        self.hash
    }
}

/// Generated manifest of source and template inputs that affect Dart output.
const CODEGEN_TOOL_FINGERPRINT_INPUT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/codegen_tool_fingerprint.txt"));

/// Hashes UTF-8 text using Dust's stable cache hash algorithm.
pub(crate) fn hash_text(text: &str) -> u64 {
    hash_bytes(text.as_bytes())
}

/// Hashes raw bytes using FNV-1a.
fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 1469598103934665603_u64;
    update_hash_bytes(&mut hash, bytes);
    hash
}

/// Mixes bytes into an existing FNV-1a hash state.
fn update_hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = (*hash).wrapping_mul(1099511628211);
    }
}

/// Computes the codegen tool hash for the selected plugin registry mode.
pub(crate) fn codegen_tool_hash_for_selection(selection: RegistrySelection) -> CodegenToolHash {
    codegen_tool_hash_for_selection_with_input(selection, CODEGEN_TOOL_FINGERPRINT_INPUT)
}

/// Computes a codegen hash from an explicit fingerprint manifest.
fn codegen_tool_hash_for_selection_with_input(
    selection: RegistrySelection,
    fingerprint_input: &str,
) -> CodegenToolHash {
    let mut hash = 1469598103934665603_u64;
    let cache_salt = selection.cache_salt();
    update_hash_bytes(&mut hash, cache_salt.as_bytes());
    update_hash_bytes(&mut hash, b"\0");
    update_hash_bytes(&mut hash, fingerprint_input.as_bytes());
    CodegenToolHash { hash }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_hash_changes_when_generated_fingerprint_changes() {
        let original =
            codegen_tool_hash_for_selection_with_input(RegistrySelection::All, "manifest:a");
        let changed =
            codegen_tool_hash_for_selection_with_input(RegistrySelection::All, "manifest:b");

        assert_ne!(original.value(), changed.value());
    }

    #[test]
    fn tool_hash_preserves_registry_mode_salt() {
        let normal =
            codegen_tool_hash_for_selection_with_input(RegistrySelection::All, "same-manifest");
        let db_only = codegen_tool_hash_for_selection_with_input(
            RegistrySelection::for_build(crate::request::DbRequestOptions {
                only_db: true,
                offline: false,
            }),
            "same-manifest",
        );

        assert_ne!(normal.value(), db_only.value());
    }

    #[test]
    fn generated_fingerprint_manifest_covers_sources_and_templates() {
        assert!(CODEGEN_TOOL_FINGERPRINT_INPUT.contains("crates/dust_driver/src/lower.rs"));
        assert!(CODEGEN_TOOL_FINGERPRINT_INPUT.contains("crates/dust_plugin_derive/src/emit.rs"));
        assert!(
            CODEGEN_TOOL_FINGERPRINT_INPUT
                .contains("crates/dust_plugin_derive/src/features/templates/debug_fields.jinja")
        );
        assert!(
            CODEGEN_TOOL_FINGERPRINT_INPUT
                .contains("crates/dust_http_client_plugin/src/plugin/emit/test_file.rs")
        );
        assert!(CODEGEN_TOOL_FINGERPRINT_INPUT.contains("crates/dust_dart_syntax/src/lib.rs"));
    }
}
