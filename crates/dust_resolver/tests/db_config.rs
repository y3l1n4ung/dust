//! Integration tests for resolver-normalized database configuration.

use dust_ir::{
    BuiltinType, DbConfigIr, DbDriverIr, DbRenameRuleIr, NormalizedConfigIr, QueryFunctionIr,
};
use dust_parser_dart::{ParseBackend, ParseOptions};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_resolver::{SymbolCatalog, resolve_library};
use dust_text::{FileId, SourceText};

#[test]
fn normalizes_database_sqlx_and_query_configs() {
    let source = SourceText::new(
        FileId::new(274),
        r#"
part 'app.g.dart';

@Database(driver: Driver.postgres, migrations: 'db/migrations')
@Sqlx(renameAll: SqlxRename.snakeCase)
class AppDatabase {
  @Sqlx(rename: 'user_id', skip: true)
  String userId;

  @Query(r'''SELECT id FROM users WHERE id = $1''')
  Future<Result<List<UserRow>, SqlxError>> findUsers();
}
"#,
    );
    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);

    let mut catalog = SymbolCatalog::new();
    catalog.register_config("Database", "dust_dart::Database");
    catalog.register_config("Sqlx", "dust_dart::Sqlx");
    catalog.register_config("Query", "dust_dart::Query");
    let resolved = resolve_library(
        FileId::new(274),
        "lib/app.dart",
        "lib/app.g.dart",
        &parsed.library,
        &catalog,
    );
    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );

    let class = &resolved.library.classes[0];
    let Some(NormalizedConfigIr::Db(DbConfigIr::Database(database))) =
        class.configs[0].normalized.as_ref()
    else {
        panic!("expected normalized Database configuration");
    };
    assert_eq!(database.driver, DbDriverIr::Postgres);
    assert_eq!(database.migrations, "db/migrations");

    let Some(NormalizedConfigIr::Db(DbConfigIr::Sqlx(sqlx))) = class.configs[1].normalized.as_ref()
    else {
        panic!("expected normalized Sqlx configuration");
    };
    assert_eq!(sqlx.rename_all, Some(DbRenameRuleIr::Snake));

    let Some(NormalizedConfigIr::Db(DbConfigIr::Sqlx(field_sqlx))) =
        class.fields[0].configs[0].normalized.as_ref()
    else {
        panic!("expected normalized field Sqlx configuration");
    };
    assert_eq!(field_sqlx.rename.as_deref(), Some("user_id"));
    assert!(field_sqlx.skip);

    let Some(NormalizedConfigIr::Db(DbConfigIr::Query(query))) =
        class.methods[0].configs[0].normalized.as_ref()
    else {
        panic!("expected normalized Query configuration");
    };
    assert_eq!(query.sql, "SELECT id FROM users WHERE id = $1");
    assert!(query.sql_source_static);
}

#[test]
fn marks_sqlx_try_from_converter_for_lowering_diagnostics() {
    let source = SourceText::new(
        FileId::new(276),
        r#"
part 'user.g.dart';

@Derive([FromRow()])
class UserRow {
  @Sqlx(tryFrom: const UserIdCodec())
  final UserId id;

  const UserRow(this.id);
}

final class UserId {
  final int value;

  const UserId(this.value);
}

final class UserIdCodec {
  const UserIdCodec();
}
"#,
    );
    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);

    let mut catalog = SymbolCatalog::new();
    catalog.register_trait("FromRow", "dust_dart::FromRow");
    catalog.register_config("Sqlx", "dust_dart::Sqlx");
    let resolved = resolve_library(
        FileId::new(276),
        "lib/user.dart",
        "lib/user.g.dart",
        &parsed.library,
        &catalog,
    );

    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    let row_class = resolved
        .library
        .classes
        .iter()
        .find(|class| class.name == "UserRow")
        .expect("expected row class");
    let converter_class = resolved
        .library
        .classes
        .iter()
        .find(|class| class.name == "UserIdCodec")
        .expect("expected converter class");
    let value_class = resolved
        .library
        .classes
        .iter()
        .find(|class| class.name == "UserId")
        .expect("expected value class");
    let Some(NormalizedConfigIr::Db(DbConfigIr::Sqlx(sqlx))) =
        row_class.fields[0].configs[0].normalized.as_ref()
    else {
        panic!("expected normalized field Sqlx configuration");
    };

    assert_eq!(sqlx.try_from_source.as_deref(), Some("const UserIdCodec()"));
    assert_eq!(sqlx.try_from_class_name.as_deref(), Some("UserIdCodec"));
    assert!(row_class.requires_lowering_diagnostics);
    assert!(converter_class.requires_lowering_diagnostics);
    assert!(!value_class.requires_lowering_diagnostics);
}

#[test]
fn normalizes_standalone_query_calls_into_ir() {
    let source = SourceText::new(
        FileId::new(275),
        r#"
Future<UserRow?> loadUser(DatabaseExecutor db, int id) {
  return queryAs<UserRow>(
    r'SELECT id, name FROM users WHERE id = $1',
    [id],
  ).fetchOptional(db);
}

Future<int> countUsers(DatabaseExecutor db) {
  return queryScalar<int>(
    'SELECT COUNT(*) FROM users',
    const <Object?>[],
  ).fetchOne(db);
}
"#,
    );
    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);

    let resolved = resolve_library(
        FileId::new(275),
        "lib/queries.dart",
        "lib/queries.g.dart",
        &parsed.library,
        &SymbolCatalog::new(),
    );

    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    assert_eq!(resolved.library.query_calls.len(), 2);

    let user_query = &resolved.library.query_calls[0];
    assert_eq!(user_query.function, QueryFunctionIr::As);
    assert!(user_query.type_arg.as_ref().unwrap().is_named("UserRow"));
    assert_eq!(user_query.type_arg_source.as_deref(), Some("UserRow"));
    assert_eq!(user_query.sql, "SELECT id, name FROM users WHERE id = $1");
    assert!(user_query.sql_source_static);
    assert_eq!(user_query.parameter_count, 1);
    assert!(user_query.params_source_is_list);
    assert_eq!(user_query.fetch_method.as_deref(), Some("fetchOptional"));
    assert_eq!(user_query.span.file_id, FileId::new(275));

    let count_query = &resolved.library.query_calls[1];
    assert_eq!(count_query.function, QueryFunctionIr::Scalar);
    assert!(
        count_query
            .type_arg
            .as_ref()
            .unwrap()
            .is_builtin(BuiltinType::Int)
    );
    assert_eq!(count_query.fetch_method.as_deref(), Some("fetchOne"));
    assert_eq!(count_query.parameter_count, 0);
}
