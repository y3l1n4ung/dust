//! Integration tests for resolver-normalized database configuration.

use dust_ir::{DbConfigIr, DbDriverIr, DbRenameRuleIr, NormalizedConfigIr};
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
