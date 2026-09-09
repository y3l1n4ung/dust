use dust_plugin_api::LibraryAnalysisSnapshot;
use tempfile::tempdir;

use super::{CacheEntry, WorkspaceCache, cache_dir_path};

/// Writes a minimal pubspec so the temporary directory resembles a package.
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
fn flush_skips_clean_cache() {
    let root = tempdir().unwrap();
    write_pubspec(root.path());

    let mut cache = WorkspaceCache::load(root.path()).unwrap();
    let cache_path = cache.path().to_path_buf();
    cache.flush().unwrap();

    assert!(!cache_path.exists());
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
            auxiliary_output_paths: Vec::new(),
            suppress_primary_output: false,
            workspace_analysis_hash: 0,
            analysis_snapshot: LibraryAnalysisSnapshot::default(),
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
            auxiliary_output_paths: Vec::new(),
            suppress_primary_output: false,
            workspace_analysis_hash: 0,
            analysis_snapshot: LibraryAnalysisSnapshot::default(),
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
            auxiliary_output_paths: Vec::new(),
            suppress_primary_output: false,
            workspace_analysis_hash: 0,
            analysis_snapshot: LibraryAnalysisSnapshot::default(),
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
            auxiliary_output_paths: Vec::new(),
            suppress_primary_output: false,
            workspace_analysis_hash: 0,
            analysis_snapshot: LibraryAnalysisSnapshot::default(),
        },
    );
    cache.flush().unwrap();

    assert!(cache_dir_path(root.path()).exists());
    assert!(WorkspaceCache::delete_storage(root.path()).unwrap());
    assert!(!cache_dir_path(root.path()).exists());
    assert!(!WorkspaceCache::delete_storage(root.path()).unwrap());
}
