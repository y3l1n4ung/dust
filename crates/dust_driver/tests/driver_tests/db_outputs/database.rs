use std::fs;

use dust_driver::{BuildRequest, DbRequestOptions, run_build};

use super::helpers::{write_db_workspace, write_row_only_workspace};
use crate::support::{generated_output, make_workspace};

#[test]
fn db_build_writes_database_without_from_row_mapper() {
    let workspace = make_workspace();
    write_db_workspace(workspace.path(), false);

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
    assert_eq!(
        output,
        generated_output(
            r#"part of 'app_database.dart';

extension UserProfileFromRow on UserProfile {
  static UserProfile fromRow(Row row) {
    return UserProfile(
      id: row.read<int>('id'),
      name: row.read<String>('display_name'),
      bio: row.readNullable<Object?>('bio') == null ? '' : row.read<String>('bio'),
    );
  }
}

final bool _$userProfileFromRowRegistered = registerRowMapper<UserProfile>(UserProfileFromRow.fromRow);

final class _$AppDatabase implements AppDatabase {
  _$AppDatabase._(this.pool);

  factory _$AppDatabase.open(String path) {
    final pool = Sqlite3Driver.open(
      path,
      migrations: _$appDatabaseMigrations,
    );
    return _$AppDatabase._(pool);
  }

  @override
  final Pool pool;
}

const Map<String, String> _$appDatabaseMigrations = <String, String>{
  '0001_init.sql': 'CREATE TABLE users (\n  id INTEGER PRIMARY KEY,\n  display_name TEXT NOT NULL,\n  bio TEXT NOT NULL DEFAULT \'\'\n);\n',
};
"#
        )
    );
}

#[test]
fn normal_build_writes_from_row_trait_without_database_output() {
    let workspace = make_workspace();
    write_row_only_workspace(workspace.path());

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let output = fs::read_to_string(workspace.path().join("lib/user_row.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_eq!(
        output,
        generated_output(
            r#"part of 'user_row.dart';

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
}
