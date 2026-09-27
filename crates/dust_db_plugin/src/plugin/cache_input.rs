use std::path::Path;

use dust_plugin_api::LibraryAnalysisSnapshot;

use super::{
    analysis::{DATABASES_KEY, UNIT},
    validate::schema_hash,
};

/// Returns whether a cached library snapshot still matches its migration files.
pub fn database_migration_cache_matches(
    package_root: &Path,
    snapshot: &LibraryAnalysisSnapshot,
) -> bool {
    snapshot
        .string_set(DATABASES_KEY)
        .unwrap_or_default()
        .iter()
        .all(|value| {
            let mut fields = value.split(UNIT);
            let Some(migrations) = fields.nth(3) else {
                return false;
            };
            let Some(expected) = fields.next() else {
                return false;
            };
            schema_hash(&package_root.join(migrations)).is_ok_and(|actual| actual == expected)
        })
}
