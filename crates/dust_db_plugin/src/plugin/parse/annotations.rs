use dust_dart_emit::apply_rename_rule;
use dust_ir::{ConfigApplicationIr, SerdeRenameRuleIr, SymbolId};
use dust_plugin_api::short_symbol_name;

use crate::plugin::{
    constants::{DAO, DATABASE, SQLX, SQLX_DAO, SQLX_DATABASE},
    model::{DbDriver, SqlxConfig, SqlxRenameRule},
};

pub(crate) fn config_name(symbol: &SymbolId) -> &str {
    short_symbol_name(&symbol.0)
}

pub(crate) fn has_config(configs: &[ConfigApplicationIr], expected: &str) -> bool {
    configs
        .iter()
        .any(|config| config_name(&config.symbol) == expected)
}

pub(crate) fn sqlx_config(configs: &[ConfigApplicationIr]) -> SqlxConfig {
    let mut out = SqlxConfig::default();
    for config in configs {
        if config_name(&config.symbol) != SQLX {
            continue;
        }
        for (key, value) in config.named_arguments() {
            match key {
                "rename" => out.rename = config.named_string("rename"),
                "renameAll" => {
                    out.rename_all = config
                        .named_member("renameAll")
                        .as_deref()
                        .and_then(parse_rename_rule);
                }
                "flatten" => out.flatten = config.named_bool("flatten").unwrap_or(out.flatten),
                "defaultValue" => out.default_value_source = Some(value.trim().to_owned()),
                "skip" => out.skip = config.named_bool("skip").unwrap_or(out.skip),
                "json" => out.json = config.named_bool("json").unwrap_or(out.json),
                "tryFrom" => out.try_from_source = Some(value.trim().to_owned()),
                _ => {}
            }
        }
    }
    out
}

pub(crate) fn effective_column_name(
    class_config: &SqlxConfig,
    field_name: &str,
    field_config: &SqlxConfig,
) -> String {
    if let Some(rename) = &field_config.rename {
        return rename.clone();
    }
    match class_config.rename_all {
        Some(rule) => apply_rename_rule(field_name, rename_to_serde(rule)),
        None => field_name.to_owned(),
    }
}

pub(super) struct DatabaseConfig {
    pub(super) driver: DbDriver,
    pub(super) migrations: String,
}

pub(super) fn parse_database_config(config: &ConfigApplicationIr) -> Option<DatabaseConfig> {
    let mut driver = DbDriver::Sqlite3;
    let mut migrations = "./migrations".to_owned();
    if let Some(parsed) = config
        .named_member("driver")
        .as_deref()
        .and_then(parse_driver)
    {
        driver = parsed;
    }
    if let Some(parsed) = config
        .named_member("type")
        .as_deref()
        .and_then(parse_database_type)
    {
        driver = parsed;
    }
    if let Some(parsed) = config.named_string("migrations") {
        migrations = parsed;
    }
    Some(DatabaseConfig { driver, migrations })
}

pub(super) fn is_database_config(name: &str) -> bool {
    matches!(name, DATABASE | SQLX_DATABASE)
}

pub(super) fn is_dao_config(name: &str) -> bool {
    matches!(name, DAO | SQLX_DAO)
}

fn rename_to_serde(rule: SqlxRenameRule) -> SerdeRenameRuleIr {
    match rule {
        SqlxRenameRule::Lower => SerdeRenameRuleIr::LowerCase,
        SqlxRenameRule::Upper => SerdeRenameRuleIr::UpperCase,
        SqlxRenameRule::Pascal => SerdeRenameRuleIr::PascalCase,
        SqlxRenameRule::Camel => SerdeRenameRuleIr::CamelCase,
        SqlxRenameRule::Snake => SerdeRenameRuleIr::SnakeCase,
        SqlxRenameRule::ScreamingSnake => SerdeRenameRuleIr::ScreamingSnakeCase,
        SqlxRenameRule::Kebab => SerdeRenameRuleIr::KebabCase,
        SqlxRenameRule::ScreamingKebab => SerdeRenameRuleIr::ScreamingKebabCase,
    }
}

fn parse_driver(source: &str) -> Option<DbDriver> {
    match source.trim().rsplit('.').next()? {
        "sqlite3" => Some(DbDriver::Sqlite3),
        "postgres" => Some(DbDriver::Postgres),
        _ => None,
    }
}

fn parse_database_type(source: &str) -> Option<DbDriver> {
    match source.trim().rsplit('.').next()? {
        "sqlite" | "sqlite3" => Some(DbDriver::Sqlite3),
        "postgres" => Some(DbDriver::Postgres),
        _ => None,
    }
}

fn parse_rename_rule(source: &str) -> Option<SqlxRenameRule> {
    match source.trim().rsplit('.').next()? {
        "lowerCase" => Some(SqlxRenameRule::Lower),
        "upperCase" => Some(SqlxRenameRule::Upper),
        "pascalCase" => Some(SqlxRenameRule::Pascal),
        "camelCase" => Some(SqlxRenameRule::Camel),
        "snakeCase" => Some(SqlxRenameRule::Snake),
        "screamingSnakeCase" => Some(SqlxRenameRule::ScreamingSnake),
        "kebabCase" => Some(SqlxRenameRule::Kebab),
        "screamingKebabCase" => Some(SqlxRenameRule::ScreamingKebab),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use dust_ir::{ConfigApplicationIr, SpanIr, SymbolId};
    use dust_text::{FileId, TextRange};

    use super::*;

    fn span() -> SpanIr {
        SpanIr::new(FileId::new(1), TextRange::new(0_u32, 1_u32))
    }

    fn config(symbol: &str, args: Option<&str>) -> ConfigApplicationIr {
        ConfigApplicationIr {
            symbol: SymbolId::new(symbol),
            arguments_source: args.map(str::to_owned),
            span: span(),
        }
    }

    #[test]
    fn parses_sqlx_config_and_effective_column_rules() {
        let config = sqlx_config(&[
            config("other::Sqlx", Some("(rename: 'ignored')")),
            config(
                "dust_db_annotation::Sqlx",
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
            parse_database_type("SqlxDatabaseType.sqlite"),
            Some(DbDriver::Sqlite3)
        );
        assert_eq!(
            parse_database_type("SqlxDatabaseType.sqlite3"),
            Some(DbDriver::Sqlite3)
        );
        assert_eq!(
            parse_database_type("SqlxDatabaseType.postgres"),
            Some(DbDriver::Postgres)
        );
        assert_eq!(parse_database_type("SqlxDatabaseType.mysql"), None);

        let db_config = parse_database_config(&config(
            "dust_db_annotation::SqlxDatabase",
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
}
