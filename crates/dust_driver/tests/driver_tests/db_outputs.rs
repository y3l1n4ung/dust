use std::fs;

use dust_driver::{BuildRequest, CheckRequest, DbRequestOptions, run_build, run_check};

use super::support::{generated_output, make_workspace, write_file};

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
      bio: row.readOrNull<Object?>('bio') == null ? '' : row.read<String>('bio'),
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

#[test]
fn db_check_rejects_sql_variable() {
    assert_static_sql_rejected(
        "final sql = 'SELECT id FROM users';\n\
         return queryRaw(sql, []).fetch(this);",
    );
}

#[test]
fn db_check_rejects_const_sql_variable() {
    assert_static_sql_rejected(
        "const sql = 'SELECT id FROM users';\n\
         return queryRaw(sql, []).fetch(this);",
    );
}

#[test]
fn db_check_rejects_interpolated_sql_literal() {
    assert_static_sql_rejected(
        "const table = 'users';\n\
         return queryRaw('SELECT id FROM $table', []).fetch(this);",
    );
}

#[test]
fn db_check_rejects_concatenated_sql_literals() {
    assert_static_sql_rejected("return queryRaw('SELECT id ' 'FROM users', []).fetch(this);");
}

#[test]
fn db_check_rejects_non_list_query_parameters() {
    let workspace = make_workspace();
    write_static_sql_validation_workspace(
        workspace.path(),
        "return queryRaw('SELECT id FROM users WHERE id = 1', params).fetch(this);",
    );

    let result = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });

    assert!(result.has_errors());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("query parameters must be a List literal")),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn db_check_accepts_runtime_values_inside_parameter_list_literal() {
    let workspace = make_workspace();
    write_static_sql_validation_workspace(
        workspace.path(),
        "return queryRaw(r'SELECT id FROM users WHERE id = $1', [params.first]).fetch(this);",
    );

    let result = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
}

#[test]
fn db_check_accepts_raw_multiline_sql_literal() {
    let workspace = make_workspace();
    write_static_sql_validation_workspace(
        workspace.path(),
        "return queryRaw(r'''\nSELECT id\nFROM users\nWHERE id = $1\n''', [params.first]).fetch(this);",
    );

    let result = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
}

#[test]
fn db_check_rejects_unknown_table_via_sqlx() {
    assert_sqlx_rejected(
        "return queryRaw('SELECT id FROM missing_users', []).fetch(this);",
        "SQLx rejected",
    );
}

#[test]
fn db_check_rejects_unknown_column_via_sqlx() {
    assert_sqlx_rejected(
        "return queryRaw('SELECT missing_id FROM users', []).fetch(this);",
        "SQLx rejected",
    );
}

#[test]
fn db_check_rejects_scalar_query_returning_multiple_columns() {
    assert_sqlx_rejected(
        "return queryScalar<int>('SELECT id, name FROM users', []).fetchOne(this);",
        "must return exactly one scalar column",
    );
}

#[test]
fn db_check_rejects_from_row_query_missing_required_column() {
    let workspace = make_workspace();
    write_query_validation_workspace(
        workspace.path(),
        "Future<List<UserRow>> allUsers() {\n\
           return queryAs<UserRow>('SELECT id FROM users', []).fetchAll(this);\n\
         }",
    );

    let result = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });

    assert!(result.has_errors());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("does not return required column `name` for row `UserRow`")),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn db_check_rejects_placeholder_count_mismatch_before_runtime() {
    assert_sqlx_rejected(
        "return queryRaw(r'SELECT id FROM users WHERE id = $1', []).fetch(this);",
        "query binds 0 args but SQL expects 1 parameters",
    );
}

fn assert_static_sql_rejected(query_body: &str) {
    let workspace = make_workspace();
    write_static_sql_validation_workspace(workspace.path(), query_body);

    let result = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });

    assert!(result.has_errors());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("query SQL must be a static string literal")),
        "{:?}",
        result.diagnostics
    );
}

fn assert_sqlx_rejected(query_body: &str, expected: &str) {
    let workspace = make_workspace();
    write_query_validation_workspace(
        workspace.path(),
        &format!("Future<Object?> runQuery() {{\n  {query_body}\n}}"),
    );

    let result = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });

    assert!(result.has_errors());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains(expected)),
        "{:?}",
        result.diagnostics
    );
}

fn write_static_sql_validation_workspace(root: &std::path::Path, query_body: &str) {
    write_file(
        &root.join("migrations/0001_init.sql"),
        "CREATE TABLE users (id INTEGER PRIMARY KEY);\n",
    );
    write_file(
        &root.join("lib/app_database.dart"),
        &format!(
            "import 'package:dust_db_annotation/dust_db_annotation.dart';\n\
             import 'package:dust_db_runtime/dust_db_runtime.dart';\n\
             import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';\n\
             part 'app_database.g.dart';\n\
             @Database(driver: Driver.sqlite3, migrations: './migrations')\n\
             abstract class AppDatabase {{\n\
               factory AppDatabase.open(String path) = _$AppDatabase.open;\n\
               Pool get pool;\n\
             }}\n\
             extension UserQueries on Pool {{\n\
               Future<List<Row>> rows(List<Object?> params) {{\n\
                 {query_body}\n\
               }}\n\
             }}\n"
        ),
    );
}

fn write_query_validation_workspace(root: &std::path::Path, query_methods: &str) {
    write_file(
        &root.join("migrations/0001_init.sql"),
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);\n",
    );
    write_file(
        &root.join("lib/app_database.dart"),
        &format!(
            "import 'package:dust_db_annotation/dust_db_annotation.dart';\n\
             import 'package:dust_db_runtime/dust_db_runtime.dart';\n\
             import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';\n\
             part 'app_database.g.dart';\n\
             @Derive([FromRow()])\n\
             final class UserRow {{\n\
               const UserRow({{required this.id, required this.name}});\n\
               final int id;\n\
               final String name;\n\
             }}\n\
             @Database(driver: Driver.sqlite3, migrations: './migrations')\n\
             abstract class AppDatabase {{\n\
               factory AppDatabase.open(String path) = _$AppDatabase.open;\n\
               Pool get pool;\n\
             }}\n\
             extension UserQueries on Pool {{\n\
               {query_methods}\n\
             }}\n"
        ),
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
        &root.join("lib/app_database.dart"),
        &format!(
            "import 'package:dust_db_annotation/dust_db_annotation.dart';\n\
             import 'package:dust_db_runtime/dust_db_runtime.dart';\n\
             import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';\n\
             part 'app_database.g.dart';\n\
             {derive_source}\n\
             @Derive([FromRow()])\n\
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
             @Database(driver: Driver.sqlite3, migrations: './migrations')\n\
             abstract class AppDatabase {{\n\
               factory AppDatabase.open(String path) = _$AppDatabase.open;\n\
               Pool get pool;\n\
             }}\n\
             extension UserQueries on Pool {{\n\
               Future<UserProfile?> findById(int id) => queryAs<UserProfile>(\n\
                 r'SELECT id, display_name, bio FROM users WHERE id = $1',\n\
                 [id],\n\
               ).fetchOptional(this);\n\
               Future<List<UserProfile>> all() => queryAs<UserProfile>(\n\
                 'SELECT id, display_name, bio FROM users',\n\
                 [],\n\
               ).fetchAll(this);\n\
               Future<int> count() => queryScalar<int>(\n\
                 'SELECT COUNT(*) FROM users',\n\
                 [],\n\
               ).fetchOne(this);\n\
               Future<QueryResult> rename(String name, int id) => queryExecute(\n\
                 r'UPDATE users SET display_name = $1 WHERE id = $2',\n\
                 [name, id],\n\
               ).execute(this);\n\
             }}\n"
        ),
    );
}

fn write_dao_workspace(root: &std::path::Path) {
    write_file(
        &root.join("migrations/0001_init.sql"),
        "CREATE TABLE users (\n  id INTEGER PRIMARY KEY,\n  display_name TEXT NOT NULL\n);\n",
    );
    write_file(
        &root.join("lib/app_database.dart"),
        "import 'package:dust_db_annotation/dust_db_annotation.dart';\n\
         import 'package:dust_db_runtime/dust_db_runtime.dart';\n\
         import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';\n\
         part 'app_database.g.dart';\n\
         @Derive([FromRow()])\n\
         final class UserProfile {\n\
           const UserProfile({required this.id, required this.name});\n\
           final int id;\n\
           @Sqlx(rename: 'display_name')\n\
           final String name;\n\
         }\n\
         @SqlxDatabase(type: SqlxDatabaseType.sqlite, migrations: './migrations')\n\
         abstract class AppDatabase {\n\
           factory AppDatabase.open(String path) = _$AppDatabase.open;\n\
           Pool get pool;\n\
         }\n\
         @SqlxDao()\n\
         abstract final class UserDao {\n\
           const factory UserDao(SqlxDriver db) = _$UserDao;\n\
           @Query(r'SELECT id, display_name FROM users WHERE id = $1')\n\
           Future<Result<UserProfile?, SqlxError>> findById(int id);\n\
         }\n",
    );
}

fn write_split_pipeline_workspace(root: &std::path::Path) {
    write_file(
        &root.join("migrations/0001_init.sql"),
        "CREATE TABLE users (\n  id INTEGER PRIMARY KEY,\n  display_name TEXT NOT NULL\n);\n",
    );
    write_file(
        &root.join("lib/user_profile.dart"),
        "import 'package:dust_db_annotation/dust_db_annotation.dart';\n\
         import 'package:dust_db_runtime/dust_db_runtime.dart';\n\
         part 'user_profile.g.dart';\n\
         @Derive([FromRow()])\n\
         final class UserProfile {\n\
           const UserProfile({required this.id, required this.name});\n\
           final int id;\n\
           @Sqlx(rename: 'display_name')\n\
           final String name;\n\
         }\n",
    );
    write_file(
        &root.join("lib/app_database.dart"),
        "import 'package:dust_db_annotation/dust_db_annotation.dart';\n\
         import 'package:dust_db_runtime/dust_db_runtime.dart';\n\
         import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';\n\
         part 'app_database.g.dart';\n\
         @SqlxDatabase(type: SqlxDatabaseType.sqlite, migrations: './migrations')\n\
         abstract class AppDatabase {\n\
           factory AppDatabase.open(String path) = _$AppDatabase.open;\n\
           Pool get pool;\n\
         }\n\
         @SqlxDao()\n\
         abstract final class UserDao {\n\
           const factory UserDao(SqlxDriver db) = _$UserDao;\n\
           @Query(r'SELECT COUNT(*) FROM users')\n\
           Future<Result<int, SqlxError>> count();\n\
         }\n",
    );
}

fn write_http_query_collision_workspace(root: &std::path::Path) {
    write_file(
        &root.join("lib/shopping_api.dart"),
        "import 'package:derive_serde_annotation/derive_serde_annotation.dart';\n\
         import 'package:dust_http_client_annotation/dust_http_client_annotation.dart';\n\
         part 'shopping_api.g.dart';\n\
         @Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])\n\
         class LoginRequest with _$LoginRequest {\n\
           const LoginRequest({required this.username, required this.password});\n\
           final String username;\n\
           final String password;\n\
           factory LoginRequest.fromJson(Map<String, Object?> json) => _$LoginRequestFromJson(json);\n\
         }\n\
         @HttpClient(baseUrl: 'https://example.com')\n\
         abstract interface class ShoppingApi {\n\
           factory ShoppingApi(Object dio, {String? baseUrl}) = _$ShoppingApi;\n\
           @GET('/products')\n\
           Future<List<String>> getProducts({@Query('limit') int? limit});\n\
         }\n",
    );
}

fn write_row_only_workspace(root: &std::path::Path) {
    write_file(
        &root.join("lib/user_row.dart"),
        "import 'package:dust_db_annotation/dust_db_annotation.dart';\n\
         import 'package:dust_db_runtime/dust_db_runtime.dart';\n\
         part 'user_row.g.dart';\n\
         @Derive([FromRow()])\n\
         @Sqlx(renameAll: SqlxRename.snakeCase)\n\
         final class UserProfile {\n\
           const UserProfile({required this.id, required this.name});\n\
           final int id;\n\
           @Sqlx(rename: 'display_name')\n\
           final String name;\n\
         }\n",
    );
}
