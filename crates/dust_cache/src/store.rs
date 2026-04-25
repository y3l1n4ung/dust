use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

/// The current on-disk Dust cache schema version.
pub const CACHE_SCHEMA_VERSION: u32 = 2;

/// One cached build fingerprint for a source library.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheEntry {
    /// The current source file fingerprint.
    pub source_hash: u64,
    /// The current package configuration fingerprint.
    pub package_config_hash: u64,
    /// The current Dust code generation fingerprint.
    #[serde(default)]
    pub tool_hash: u64,
    /// The expected generated output fingerprint for the current source state.
    pub expected_output_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CacheFile {
    schema_version: u32,
    entries: BTreeMap<String, CacheEntry>,
}

/// The persistent cache for one Dart workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceCache {
    path: PathBuf,
    entries: BTreeMap<String, CacheEntry>,
}

impl WorkspaceCache {
    /// Loads the workspace cache from `.dart_tool/dust/build_cache_v1.json`.
    ///
    /// Missing cache files are treated as an empty cache.
    pub fn load(root: &Path) -> io::Result<Self> {
        let path = cache_file_path(root);
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(Self {
                    path,
                    entries: BTreeMap::new(),
                });
            }
            Err(error) => return Err(error),
        };

        let parsed = serde_json::from_str::<CacheFile>(&contents)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if parsed.schema_version != CACHE_SCHEMA_VERSION {
            return Ok(Self {
                path,
                entries: BTreeMap::new(),
            });
        }

        Ok(Self {
            path,
            entries: parsed.entries,
        })
    }

    /// Returns the on-disk cache file path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the cache entry for one source file if present.
    pub fn get(&self, root: &Path, source_path: &Path) -> Option<&CacheEntry> {
        self.entries.get(&cache_key(root, source_path))
    }

    /// Inserts or replaces the cache entry for one source file.
    pub fn insert(&mut self, root: &Path, source_path: &Path, entry: CacheEntry) {
        self.entries.insert(cache_key(root, source_path), entry);
    }

    /// Removes the cache entry for one source file.
    pub fn remove(&mut self, root: &Path, source_path: &Path) {
        self.entries.remove(&cache_key(root, source_path));
    }

    /// Persists the current cache contents to disk.
    pub fn flush(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = CacheFile {
            schema_version: CACHE_SCHEMA_VERSION,
            entries: self.entries.clone(),
        };
        let json = serde_json::to_string_pretty(&file)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        fs::write(&self.path, json)
    }

    /// Deletes the entire Dust cache storage directory under `.dart_tool`.
    ///
    /// Returns `Ok(true)` when the directory existed and was removed.
    /// Returns `Ok(false)` when no Dust cache storage was present.
    pub fn delete_storage(root: &Path) -> io::Result<bool> {
        let path = cache_dir_path(root);
        match fs::remove_dir_all(&path) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}

fn cache_key(root: &Path, source_path: &Path) -> String {
    source_path
        .strip_prefix(root)
        .unwrap_or(source_path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn cache_file_path(root: &Path) -> PathBuf {
    cache_dir_path(root).join("build_cache_v1.json")
}

fn cache_dir_path(root: &Path) -> PathBuf {
    root.join(".dart_tool/dust")
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{CacheEntry, WorkspaceCache, cache_dir_path};

    fn write_pubspec(root: &std::path::Path) {
        std::fs::write(root.join("pubspec.yaml"), "name: dust_test\n").unwrap();
    }

    #[test]
    fn loads_empty_cache_when_file_is_missing() {
        let root = tempdir().unwrap();
        write_pubspec(root.path());

        let cache = WorkspaceCache::load(root.path()).unwrap();

        assert!(
            cache
                .path()
                .ends_with(std::path::Path::new(".dart_tool/dust/build_cache_v1.json"))
        );
        assert!(
            cache
                .get(root.path(), &root.path().join("lib/user.dart"))
                .is_none()
        );
    }

    #[test]
    fn round_trips_entries_using_workspace_relative_keys() {
        let root = tempdir().unwrap();
        write_pubspec(root.path());
        let source_path = root.path().join("lib/models/user.dart");

        let mut cache = WorkspaceCache::load(root.path()).unwrap();
        cache.insert(
            root.path(),
            &source_path,
            CacheEntry {
                source_hash: 1,
                package_config_hash: 2,
                tool_hash: 3,
                expected_output_hash: 4,
            },
        );
        cache.flush().unwrap();

        let reloaded = WorkspaceCache::load(root.path()).unwrap();
        assert_eq!(
            reloaded.get(root.path(), &source_path),
            Some(&CacheEntry {
                source_hash: 1,
                package_config_hash: 2,
                tool_hash: 3,
                expected_output_hash: 4,
            })
        );
    }

    #[test]
    fn removes_entries_cleanly() {
        let root = tempdir().unwrap();
        write_pubspec(root.path());
        let source_path = root.path().join("lib/user.dart");

        let mut cache = WorkspaceCache::load(root.path()).unwrap();
        cache.insert(
            root.path(),
            &source_path,
            CacheEntry {
                source_hash: 10,
                package_config_hash: 20,
                tool_hash: 30,
                expected_output_hash: 40,
            },
        );
        cache.remove(root.path(), &source_path);
        cache.flush().unwrap();

        let reloaded = WorkspaceCache::load(root.path()).unwrap();
        assert!(reloaded.get(root.path(), &source_path).is_none());
    }

    #[test]
    fn deletes_storage_directory_when_requested() {
        let root = tempdir().unwrap();
        write_pubspec(root.path());
        let mut cache = WorkspaceCache::load(root.path()).unwrap();
        cache.insert(
            root.path(),
            &root.path().join("lib/user.dart"),
            CacheEntry {
                source_hash: 1,
                package_config_hash: 2,
                tool_hash: 3,
                expected_output_hash: 4,
            },
        );
        cache.flush().unwrap();

        assert!(cache_dir_path(root.path()).exists());
        assert!(WorkspaceCache::delete_storage(root.path()).unwrap());
        assert!(!cache_dir_path(root.path()).exists());
        assert!(!WorkspaceCache::delete_storage(root.path()).unwrap());
    }
}
