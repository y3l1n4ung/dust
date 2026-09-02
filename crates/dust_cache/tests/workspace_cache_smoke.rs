//! Integration smoke tests for the public workspace cache API.

use std::{
    fs,
    path::{Path, PathBuf},
};

use dust_cache::{CACHE_SCHEMA_VERSION, CacheEntry, WorkspaceCache};
use dust_plugin_api::WorkspaceAnalysisBuilder;
use tempfile::tempdir;

#[test]
fn workspace_cache_persists_updates_and_cached_analysis() {
    let root = tempdir().unwrap();
    let source_path = root.path().join("lib/models/user.dart");
    let entry = cache_entry();

    let mut cache = WorkspaceCache::load(root.path()).unwrap();
    assert!(cache.path().ends_with(cache_file_suffix()));
    assert_eq!(cache.get(root.path(), &source_path), None);

    cache.insert(root.path(), &source_path, entry.clone());
    assert_eq!(cache.get(root.path(), &source_path), Some(&entry));
    cache.flush().unwrap();
    assert_eq!(
        flushed_schema_version(cache.path()),
        u64::from(CACHE_SCHEMA_VERSION)
    );

    let reloaded = WorkspaceCache::load(root.path()).unwrap();
    let reloaded_entry = reloaded
        .get(root.path(), &source_path)
        .expect("cache entry must persist after flush");
    assert_eq!(reloaded_entry, &entry);
    assert_eq!(
        reloaded_entry
            .analysis_snapshot
            .string_set("dust_test.routes.v1"),
        Some(&["DashboardRoute".to_owned(), "LoginRoute".to_owned()][..])
    );

    let mut updated = reloaded.clone();
    updated.remove(root.path(), &source_path);
    updated.flush().unwrap();
    let after_remove = WorkspaceCache::load(root.path()).unwrap();
    assert_eq!(after_remove.get(root.path(), &source_path), None);

    assert!(WorkspaceCache::delete_storage(root.path()).unwrap());
    assert!(!WorkspaceCache::delete_storage(root.path()).unwrap());
}

fn cache_entry() -> CacheEntry {
    let mut analysis = WorkspaceAnalysisBuilder::default();
    analysis.add_string_set_value("dust_test.routes.v1", "LoginRoute");
    analysis.add_string_set_value("dust_test.routes.v1", "DashboardRoute");

    CacheEntry {
        source_hash: 11,
        package_config_hash: 22,
        tool_hash: 33,
        expected_output_hash: 44,
        auxiliary_output_paths: vec![PathBuf::from("lib/models/user.extra.g.dart")],
        suppress_primary_output: false,
        workspace_analysis_hash: 0,
        analysis_snapshot: analysis.snapshot(),
    }
}

fn cache_file_suffix() -> &'static Path {
    Path::new(".dart_tool/dust/build_cache_v1.json")
}

fn flushed_schema_version(path: &Path) -> u64 {
    let json = fs::read_to_string(path).unwrap();
    let parsed = serde_json::from_str::<serde_json::Value>(&json).unwrap();
    parsed["schema_version"]
        .as_u64()
        .expect("cache schema version must be numeric")
}
