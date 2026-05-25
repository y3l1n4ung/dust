use std::fs;

use dust_driver::{BuildRequest, CheckRequest, DbRequestOptions, run_build, run_check};

use super::support::{generated_output, make_workspace, write_file};

#[test]
fn build_writes_db_repository_and_row_mapper() {
    let workspace = make_workspace();
    write_db_workspace(workspace.path(), false);

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let output = fs::read_to_string(workspace.path().join("lib/user_repository.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_eq!(
        output,
        generated_output(
            r#"part of 'user_repository.dart';

extension UserProfileFromRow on UserProfile {
  static UserProfile fromRow(Map<String, Object?> row) => UserProfile(
    id: row['id'] as int,
    name: row['display_name'] as String,
    bio: row.containsKey('bio') ? row['bio'] as String : '',
  );
}

final class _$UserRepository implements UserRepository {
  final dynamic _db;

  _$UserRepository(this._db);

  @override
  Future<UserProfile?> findById(int id) async {
    final rows = await _db.rawQuery(
      'SELECT id, display_name, bio FROM users WHERE id = ?',
      <Object?>[id],
    );
    if (rows.isEmpty) return null;
    return UserProfileFromRow.fromRow(rows.first);
  }

  @override
  Future<List<UserProfile>> all() async {
    final rows = await _db.rawQuery(
      'SELECT id, display_name, bio FROM users',
      const <Object?>[],
    );
    return rows.map(UserProfileFromRow.fromRow).toList();
  }

  @override
  Future<int> count() async {
    final rows = await _db.rawQuery(
      'SELECT COUNT(*) FROM users',
      const <Object?>[],
    );
    return rows.first.values.first as int;
  }

  @override
  Future<void> rename(String name, int id) async {
    await _db.execute(
      'UPDATE users SET display_name = ? WHERE id = ?',
      <Object?>[name, id],
    );
  }
}
"#
        )
    );
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
    let output = fs::read_to_string(workspace.path().join("lib/user_repository.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(output.contains("final class _$UserRepository"));
    assert!(!output.contains("mixin _$DebugLabel"));
    assert!(!output.contains("String toString()"));
}

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

fn write_db_workspace(root: &std::path::Path, include_derive: bool) {
    write_file(
        &root.join("migrations/0001_init.sql"),
        "CREATE TABLE users (\n  id INTEGER PRIMARY KEY,\n  display_name TEXT NOT NULL,\n  bio TEXT NOT NULL DEFAULT ''\n);\n",
    );
    let derive_source = if include_derive {
        "@ToString()\nfinal class DebugLabel {\n  const DebugLabel(this.value);\n  final String value;\n}\n"
    } else {
        ""
    };
    write_file(
        &root.join("lib/user_repository.dart"),
        &format!(
            "import 'package:dust_db/dust_db.dart';\n\
             part 'user_repository.g.dart';\n\
             {derive_source}\n\
             @FromRow()\n\
             @Sqlx(renameAll: SqlxRename.snakeCase)\n\
             final class UserProfile {{\n\
               const UserProfile({{\n\
                 required this.id,\n\
                 required this.name,\n\
                 this.bio = '',\n\
                 this.sessionActive = false,\n\
               }});\n\
               final int id;\n\
               @Sqlx(rename: 'display_name')\n\
               final String name;\n\
               final String bio;\n\
               @Sqlx(skip: true)\n\
               final bool sessionActive;\n\
             }}\n\
             @DustDb(driver: Driver.sqflite, migrations: 'migrations')\n\
             abstract interface class UserRepository {{\n\
               factory UserRepository(dynamic db) = _$UserRepository;\n\
               @Query('SELECT id, display_name, bio FROM users WHERE id = ?')\n\
               Future<UserProfile?> findById(int id);\n\
               @Query('SELECT id, display_name, bio FROM users')\n\
               Future<List<UserProfile>> all();\n\
               @Query('SELECT COUNT(*) FROM users')\n\
               Future<int> count();\n\
               @Query('UPDATE users SET display_name = ? WHERE id = ?')\n\
               Future<void> rename(String name, int id);\n\
             }}\n"
        ),
    );
}
