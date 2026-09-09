use dust_ir::{ConfigApplicationIr, SpanIr, SymbolId};
use dust_text::{FileId, TextRange};

use super::*;

fn span() -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(0_u32, 1_u32))
}

fn config(symbol: &str, args: Option<&str>) -> ConfigApplicationIr {
    ConfigApplicationIr::new(SymbolId::new(symbol), args.map(str::to_owned), span())
}

#[test]
fn parses_sqlx_config_and_effective_column_rules() {
    let config = sqlx_config(&[
        config("other::Sqlx", Some("(rename: 'ignored')")),
        config(
            "dust_dart::Sqlx",
            Some(
                "(rename: 'display_name', renameAll: SqlxRename.snakeCase, flatten: true, defaultValue: '', skip: true, json: true, tryFrom: const UserStatusFromInt(), unknown: true)",
            ),
        ),
    ]);

    assert_eq!(config.rename.as_deref(), Some("display_name"));
    assert_eq!(config.rename_all, Some(SqlxRenameRule::Snake));
    assert!(config.flatten);
    assert_eq!(config.default_value_source.as_deref(), Some("''"));
    assert!(config.skip);
    assert!(config.json);
    assert_eq!(
        config.try_from_source.as_deref(),
        Some("const UserStatusFromInt()")
    );
    assert_eq!(
        effective_column_name(&config, "createdAt", &SqlxConfig::default()),
        "created_at"
    );
    assert_eq!(
        effective_column_name(
            &config,
            "createdAt",
            &SqlxConfig {
                rename: Some("created_at_override".to_owned()),
                ..SqlxConfig::default()
            },
        ),
        "created_at_override"
    );
}

#[test]
fn parses_database_and_rename_variants() {
    assert_eq!(parse_driver("Driver.sqlite3"), Some(DbDriver::Sqlite3));
    assert_eq!(parse_driver("Driver.postgres"), Some(DbDriver::Postgres));
    assert_eq!(parse_driver("Driver.mysql"), None);
    assert_eq!(
        parse_driver("SqlxDatabaseType.sqlite"),
        Some(DbDriver::Sqlite3)
    );
    assert_eq!(
        parse_driver("SqlxDatabaseType.sqlite3"),
        Some(DbDriver::Sqlite3)
    );
    assert_eq!(
        parse_driver("SqlxDatabaseType.postgres"),
        Some(DbDriver::Postgres)
    );
    assert_eq!(parse_driver("SqlxDatabaseType.mysql"), None);

    let db_config = parse_database_config(&config(
        "dust_dart::SqlxDatabase",
        Some("(driver: Driver.postgres, migrations: './db/migrations', ignored: true)"),
    ))
    .unwrap();
    assert_eq!(db_config.driver, DbDriver::Postgres);
    assert_eq!(db_config.migrations, "./db/migrations");

    assert_eq!(
        parse_rename_rule("SqlxRename.lowerCase"),
        Some(SqlxRenameRule::Lower)
    );
    assert_eq!(
        parse_rename_rule("SqlxRename.upperCase"),
        Some(SqlxRenameRule::Upper)
    );
    assert_eq!(
        parse_rename_rule("SqlxRename.pascalCase"),
        Some(SqlxRenameRule::Pascal)
    );
    assert_eq!(
        parse_rename_rule("SqlxRename.camelCase"),
        Some(SqlxRenameRule::Camel)
    );
    assert_eq!(
        parse_rename_rule("SqlxRename.snakeCase"),
        Some(SqlxRenameRule::Snake)
    );
    assert_eq!(
        parse_rename_rule("SqlxRename.screamingSnakeCase"),
        Some(SqlxRenameRule::ScreamingSnake)
    );
    assert_eq!(
        parse_rename_rule("SqlxRename.kebabCase"),
        Some(SqlxRenameRule::Kebab)
    );
    assert_eq!(
        parse_rename_rule("SqlxRename.screamingKebabCase"),
        Some(SqlxRenameRule::ScreamingKebab)
    );
    assert_eq!(parse_rename_rule("SqlxRename.unknown"), None);
}
