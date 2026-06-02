use dust_driver::{CheckRequest, DbRequestOptions, run_check};

use crate::support::{make_workspace, write_file};

pub(crate) fn assert_static_sql_rejected(query_body: &str) {
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

pub(crate) fn assert_sqlx_rejected(query_body: &str, expected: &str) {
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

pub(crate) fn write_static_sql_validation_workspace(root: &std::path::Path, query_body: &str) {
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

pub(crate) fn write_query_validation_workspace(root: &std::path::Path, query_methods: &str) {
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

pub(crate) fn write_db_workspace(root: &std::path::Path, include_derive: bool) {
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

pub(crate) fn write_dao_workspace(root: &std::path::Path) {
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

pub(crate) fn write_split_pipeline_workspace(root: &std::path::Path) {
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

pub(crate) fn write_http_query_collision_workspace(root: &std::path::Path) {
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

pub(crate) fn write_row_only_workspace(root: &std::path::Path) {
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
