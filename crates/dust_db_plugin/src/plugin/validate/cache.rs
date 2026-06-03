use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::plugin::model::QuerySpec;

use super::query::{query_row_type, validate_placeholders};

pub(super) const QUERY_CACHE_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct QueryCache {
    pub(super) version: u32,
    pub(super) entries: Vec<QueryCacheEntry>,
}

impl Default for QueryCache {
    fn default() -> Self {
        Self {
            version: QUERY_CACHE_VERSION,
            entries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct QueryCacheEntry {
    pub(super) migrations: String,
    pub(super) schema_hash: String,
    pub(super) sql_hash: String,
    pub(super) sql: String,
    pub(super) user_parameter_count: usize,
    pub(super) expanded_parameter_count: usize,
    pub(super) fetch_mode: String,
    pub(super) row_type: Option<String>,
    pub(super) columns: Vec<String>,
}

pub(super) fn validate_from_query_cache(
    library: &dust_ir::LibraryIr,
    migrations: &str,
    schema_hash: &str,
    queries: &[QuerySpec],
    row_columns: &HashMap<String, HashSet<String>>,
) -> Result<(), String> {
    let cache_path = query_cache_path(library);
    let cache_source = fs::read_to_string(&cache_path).map_err(|error| {
        format!(
            "Dust DB offline query metadata cache is missing or unreadable at `{}`: {error}",
            cache_path.display()
        )
    })?;
    let cache: QueryCache = serde_json::from_str(&cache_source).map_err(|error| {
        format!(
            "Dust DB offline query metadata cache `{}` is invalid: {error}",
            cache_path.display()
        )
    })?;
    if cache.version != QUERY_CACHE_VERSION {
        return Err(format!(
            "Dust DB offline query metadata cache `{}` uses unsupported version {}; run `dust build --db` online first",
            cache_path.display(),
            cache.version
        ));
    }

    for query in queries {
        validate_cached_query(migrations, schema_hash, query, row_columns, &cache)?;
    }
    Ok(())
}

pub(super) fn write_query_cache(
    library: &dust_ir::LibraryIr,
    entries: Vec<QueryCacheEntry>,
) -> Result<(), String> {
    if entries.is_empty() {
        return Ok(());
    }

    let path = query_cache_path(library);
    let mut cache = fs::read_to_string(&path)
        .ok()
        .and_then(|source| serde_json::from_str::<QueryCache>(&source).ok())
        .filter(|cache| cache.version == QUERY_CACHE_VERSION)
        .unwrap_or_default();
    for entry in entries {
        cache.entries.retain(|existing| {
            !(existing.migrations == entry.migrations
                && existing.schema_hash == entry.schema_hash
                && existing.sql_hash == entry.sql_hash
                && existing.fetch_mode == entry.fetch_mode
                && existing.row_type == entry.row_type)
        });
        cache.entries.push(entry);
    }
    cache.entries.sort_by(|left, right| {
        left.migrations
            .cmp(&right.migrations)
            .then(left.schema_hash.cmp(&right.schema_hash))
            .then(left.sql_hash.cmp(&right.sql_hash))
            .then(left.fetch_mode.cmp(&right.fetch_mode))
    });

    let source = serde_json::to_string_pretty(&cache)
        .map_err(|error| format!("failed to encode Dust DB query cache: {error}"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create Dust DB query cache directory `{}`: {error}",
                parent.display()
            )
        })?;
    }
    fs::write(&path, format!("{source}\n")).map_err(|error| {
        format!(
            "failed to write Dust DB query cache `{}`: {error}",
            path.display()
        )
    })
}

pub(super) fn validate_cached_columns(
    query: &QuerySpec,
    row_columns: &HashMap<String, HashSet<String>>,
    returned_columns: &[String],
) -> Result<(), String> {
    let Some(row_type) = query_row_type(query) else {
        return Ok(());
    };
    let Some(required_columns) = row_columns.get(row_type) else {
        return Ok(());
    };
    let returned_columns = returned_columns.iter().collect::<HashSet<_>>();
    if let Some(missing) = required_columns
        .iter()
        .find(|column| !returned_columns.contains(*column))
    {
        return Err(format!(
            "cached SQL metadata for `{}` does not return required column `{missing}` for row `{row_type}`",
            query.display_name()
        ));
    }
    Ok(())
}

pub(super) fn migration_files(path: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = fs::read_dir(path)
        .map_err(|error| format!("failed to read migrations `{}`: {error}", path.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sql"))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

pub(super) fn schema_hash(migrations_path: &Path) -> Result<String, String> {
    let mut hash = StableHash::new();
    for migration in migration_files(migrations_path)? {
        let relative_path = migration
            .strip_prefix(migrations_path)
            .unwrap_or(&migration);
        hash.update(relative_path.to_string_lossy().as_bytes());
        hash.update(b"\0");
        let source = fs::read(&migration).map_err(|error| {
            format!(
                "failed to read migration `{}`: {error}",
                migration.display()
            )
        })?;
        hash.update(&source);
        hash.update(b"\0");
    }
    Ok(hash.finish_hex())
}

pub(super) fn stable_hash_hex(bytes: &[u8]) -> String {
    let mut hash = StableHash::new();
    hash.update(bytes);
    hash.finish_hex()
}

pub(super) fn query_cache_path(library: &dust_ir::LibraryIr) -> PathBuf {
    Path::new(&library.package_root).join(".dart_tool/dust/db_query_cache_v2.json")
}

fn validate_cached_query(
    migrations: &str,
    schema_hash: &str,
    query: &QuerySpec,
    row_columns: &HashMap<String, HashSet<String>>,
    cache: &QueryCache,
) -> Result<(), String> {
    let rewrite = validate_placeholders(&query.sql, query.parameter_count)?;
    let sql_hash = stable_hash_hex(query.sql.as_bytes());
    let Some(entry) = cache.entries.iter().find(|entry| {
        entry.migrations == migrations
            && entry.schema_hash == schema_hash
            && entry.sql_hash == sql_hash
            && entry.sql == query.sql
            && entry.fetch_mode == query.fetch.as_str()
            && entry.row_type == query.row_type
    }) else {
        return Err(format!(
            "Dust DB offline query metadata cache is missing entry for `{}`; run `dust build --db` online first",
            query.display_name()
        ));
    };
    if entry.user_parameter_count != query.parameter_count {
        return Err(format!(
            "cached SQL metadata for `{}` expects {} parameters but query binds {} args",
            query.display_name(),
            entry.user_parameter_count,
            query.parameter_count
        ));
    }
    if entry.expanded_parameter_count != rewrite.expanded_parameter_count() {
        return Err(format!(
            "cached SQL metadata for `{}` has stale placeholder expansion; run `dust build --db` online first",
            query.display_name()
        ));
    }
    validate_cached_columns(query, row_columns, &entry.columns)
}

struct StableHash(u64);

impl StableHash {
    const fn new() -> Self {
        Self(1469598103934665603)
    }

    fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(1099511628211);
        }
    }

    fn finish_hex(self) -> String {
        format!("{:016x}", self.0)
    }
}
