use std::fs;

use dust_driver::{BuildRequest, DbRequestOptions, run_build};

use super::helpers::{
    write_dao_workspace, write_db_workspace, write_http_query_collision_workspace,
    write_split_pipeline_workspace,
};
use crate::support::{generated_output, make_workspace};

#[test]
fn normal_build_does_not_generate_sqlx_database_or_dao_output() {
    let workspace = make_workspace();
    write_dao_workspace(workspace.path());

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let output_path = workspace.path().join("lib/app_database.g.dart");

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    let output = fs::read_to_string(output_path).unwrap();
    assert_eq!(
        output,
        generated_output(
            r#"part of 'app_database.dart';

extension UserProfileFromRow on UserProfile {
  static UserProfile fromRow(Row row) {
    return UserProfile(
      id: row.read<int>('id'),
      name: row.read<String>('display_name'),
    );
  }
}

final bool _$userProfileFromRowRegistered = registerRowMapper<UserProfile>(UserProfileFromRow.fromRow);
"#
        )
    );
    assert!(!output.contains("final class _$AppDatabase"));
    assert!(!output.contains("final class _$UserDao"));
}

#[test]
fn db_build_writes_sqlx_dao_output() {
    let workspace = make_workspace();
    write_split_pipeline_workspace(workspace.path());

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });
    let output = fs::read_to_string(workspace.path().join("lib/app_database.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(output.contains("final class _$AppDatabase"));
    assert!(output.contains("final class _$UserDao implements UserDao"));
    assert!(output.contains("Future<Result<int, SqlxError>> count"));
    assert!(!workspace.path().join("lib/user_profile.g.dart").exists());
    assert!(!output.contains("UserProfileFromRow"));
}

#[test]
fn db_build_ignores_non_db_derive_members_in_query_collision_files() {
    let workspace = make_workspace();
    write_http_query_collision_workspace(workspace.path());

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });
    let output_path = workspace.path().join("lib/shopping_api.g.dart");

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(
        result.diagnostics.iter().all(|diagnostic| !diagnostic
            .message
            .contains("unknown derive trait or config")),
        "{:?}",
        result.diagnostics
    );
    assert!(!output_path.exists());
}

#[test]
fn db_only_build_does_not_emit_derive_output() {
    let workspace = make_workspace();
    write_db_workspace(workspace.path(), true);

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });
    let output = fs::read_to_string(workspace.path().join("lib/app_database.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(output.contains("final class _$AppDatabase"));
    assert!(!output.contains("mixin _$DebugLabel"));
    assert!(!output.contains("String toString()"));
}

#[test]
fn normal_build_preserves_existing_database_output() {
    let workspace = make_workspace();
    write_split_pipeline_workspace(workspace.path());

    let db_build = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });
    let normal_build = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let database_output =
        fs::read_to_string(workspace.path().join("lib/app_database.g.dart")).unwrap();
    let row_output = fs::read_to_string(workspace.path().join("lib/user_profile.g.dart")).unwrap();

    assert!(!db_build.has_errors(), "{:?}", db_build.diagnostics);
    assert!(!normal_build.has_errors(), "{:?}", normal_build.diagnostics);
    assert!(database_output.contains("final class _$AppDatabase"));
    assert!(database_output.contains("final class _$UserDao implements UserDao"));
    assert!(database_output.contains("Sqlite3Driver.open"));
    assert!(row_output.contains("extension UserProfileFromRow on UserProfile"));
}
