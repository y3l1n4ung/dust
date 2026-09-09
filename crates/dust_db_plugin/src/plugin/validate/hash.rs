//! Stable hashing for the offline query cache.
//!
//! The cache is committed, so its keys have to be identical on every machine
//! and every run. `DefaultHasher` is not: it is explicitly allowed to change
//! between releases. This is a fixed FNV-1a instead, spelled out here so the
//! hash a checkout reads is the hash the machine that wrote it computed.

use std::{fs, path::Path};

use crate::plugin::migrations::applied_migration_files;

/// Computes a stable schema hash from migration file names and contents.
pub(super) fn schema_hash(migrations_path: &Path) -> Result<String, String> {
    let mut hash = StableHash::new();
    for migration in applied_migration_files(migrations_path)? {
        hash.update(migration.name.as_bytes());
        hash.update(b"\0");
        let source = fs::read(&migration.path).map_err(|error| {
            format!(
                "failed to read migration `{}`: {error}",
                migration.path.display()
            )
        })?;
        hash.update(&source);
        hash.update(b"\0");
    }
    Ok(hash.finish_hex())
}

/// Computes a stable hexadecimal hash for cache keys.
pub(super) fn stable_hash_hex(bytes: &[u8]) -> String {
    let mut hash = StableHash::new();
    hash.update(bytes);
    hash.finish_hex()
}

/// Small deterministic FNV-1a hasher for cache keys.
struct StableHash(u64);

impl StableHash {
    /// Creates a hasher with the FNV offset basis.
    const fn new() -> Self {
        Self(1469598103934665603)
    }

    /// Adds bytes to the stable hash.
    fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(1099511628211);
        }
    }

    /// Returns the final hash as a fixed-width hex string.
    fn finish_hex(self) -> String {
        format!("{:016x}", self.0)
    }
}
