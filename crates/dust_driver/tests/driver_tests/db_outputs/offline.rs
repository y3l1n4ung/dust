use std::fs;

use dust_driver::{BuildRequest, CheckRequest, DbRequestOptions, run_build, run_check};

use super::helpers::write_db_workspace;
use crate::support::make_workspace;

#[test]
fn offline_db_check_fails_when_query_metadata_is_missing() {
    let workspace = make_workspace();
    write_db_workspace(workspace.path(), false);

    let result = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: true,
        },
    });

    assert!(result.has_errors());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("offline query metadata cache is missing")),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn online_db_build_writes_query_metadata_for_offline_check() {
    let workspace = make_workspace();
    write_db_workspace(workspace.path(), false);

    let build = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });
    let cache_source = read_query_cache(workspace.path());

    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    assert!(cache_source.contains("SELECT id, display_name, bio FROM users WHERE id = $1"));

    let check = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: true,
        },
    });

    assert!(!check.has_errors(), "{:?}", check.diagnostics);
}

#[test]
fn online_db_check_does_not_write_query_metadata() {
    let workspace = make_workspace();
    write_db_workspace(workspace.path(), false);

    let check = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });
    assert!(!check.has_errors(), "{:?}", check.diagnostics);
    assert!(query_cache_files(workspace.path()).is_empty());
}

#[test]
fn offline_db_check_rejects_unsupported_query_metadata_version() {
    let workspace = make_workspace();
    write_db_workspace(workspace.path(), false);
    // Build online first so the cache exists at the path the library reads,
    // then leave a version the current format does not accept.
    run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });
    for cache in query_cache_files(workspace.path()) {
        fs::write(&cache, r#"{"version":999,"entries":[]}"#).unwrap();
    }

    let check = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: true,
        },
    });

    assert!(check.has_errors());
    assert!(
        check
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("unsupported version 999")),
        "{:?}",
        check.diagnostics
    );
}

/// Returns every per-library DB query cache file in a workspace.
fn query_cache_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let dir = root.join(".dart_tool/dust/db_query_cache_v2");
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut paths = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

/// Returns the concatenated contents of every DB query cache file.
fn read_query_cache(root: &std::path::Path) -> String {
    query_cache_files(root)
        .into_iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .collect::<Vec<_>>()
        .join("\n")
}
