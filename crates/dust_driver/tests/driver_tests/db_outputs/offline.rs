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
    let cache = workspace
        .path()
        .join(".dart_tool/dust/db_query_cache_v2.json");
    let cache_source = fs::read_to_string(&cache).unwrap();

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
    let cache = workspace
        .path()
        .join(".dart_tool/dust/db_query_cache_v2.json");

    assert!(!check.has_errors(), "{:?}", check.diagnostics);
    assert!(!cache.exists());
}

#[test]
fn offline_db_check_rejects_unsupported_query_metadata_version() {
    let workspace = make_workspace();
    write_db_workspace(workspace.path(), false);
    let cache = workspace
        .path()
        .join(".dart_tool/dust/db_query_cache_v2.json");
    fs::create_dir_all(cache.parent().unwrap()).unwrap();
    fs::write(&cache, r#"{"version":999,"entries":[]}"#).unwrap();

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
