use dust_dart_emit::apply_rename_rule;
use dust_ir::{
    ConfigApplicationIr, DbConfigIr, DbDriverIr, DbRenameRuleIr, NormalizedConfigIr,
    SerdeRenameRuleIr, SymbolId,
};
use dust_plugin_api::short_symbol_name;

use crate::plugin::{
    constants::{DAO, DATABASE, SQLX, SQLX_DAO, SQLX_DATABASE},
    dialect::Dialect,
    model::{DbDriver, SqlxConfig, SqlxRenameRule},
};

/// Returns the short annotation name for a resolved symbol.
pub(crate) fn config_name(symbol: &SymbolId) -> &str {
    short_symbol_name(&symbol.0)
}

/// Returns true when an annotation list contains the expected short name.
pub(crate) fn has_config(configs: &[ConfigApplicationIr], expected: &str) -> bool {
    configs
        .iter()
        .any(|config| config_name(&config.symbol) == expected)
}

/// Parses all `@Sqlx` configs attached to a class or field.
pub(crate) fn sqlx_config(configs: &[ConfigApplicationIr]) -> SqlxConfig {
    let mut out = SqlxConfig::default();
    for config in configs {
        if config_name(&config.symbol) != SQLX {
            continue;
        }
        if let Some(NormalizedConfigIr::Db(DbConfigIr::Sqlx(normalized))) =
            config.normalized.as_ref()
        {
            out.rename = normalized.rename.clone();
            out.rename_all = normalized.rename_all.map(rename_from_ir);
            out.flatten = normalized.flatten;
            out.default_value_source = normalized.default_value_source.clone();
            out.skip = normalized.skip;
            out.json = normalized.json;
            out.try_from_source = normalized.try_from_source.clone();
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

/// Resolves the final SQL column name for a row field.
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

/// Parsed database annotation options.
pub(super) struct DatabaseConfig {
    /// Database driver selected by the annotation.
    pub(super) driver: DbDriver,
    /// Migration directory configured by the annotation.
    pub(super) migrations: String,
}

/// Parses `@Database` or `@SqlxDatabase` options.
pub(super) fn parse_database_config(config: &ConfigApplicationIr) -> Option<DatabaseConfig> {
    if let Some(NormalizedConfigIr::Db(DbConfigIr::Database(normalized))) =
        config.normalized.as_ref()
    {
        return Some(DatabaseConfig {
            driver: match normalized.driver {
                DbDriverIr::Sqlite3 => DbDriver::Sqlite3,
                DbDriverIr::Postgres => DbDriver::Postgres,
            },
            migrations: normalized.migrations.clone(),
        });
    }
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
        .and_then(parse_driver)
    {
        driver = parsed;
    }
    if let Some(parsed) = config.named_string("migrations") {
        migrations = parsed;
    }
    Some(DatabaseConfig { driver, migrations })
}

/// Converts normalized DB rename rules into the plugin model.
fn rename_from_ir(rule: DbRenameRuleIr) -> SqlxRenameRule {
    match rule {
        DbRenameRuleIr::Lower => SqlxRenameRule::Lower,
        DbRenameRuleIr::Upper => SqlxRenameRule::Upper,
        DbRenameRuleIr::Pascal => SqlxRenameRule::Pascal,
        DbRenameRuleIr::Camel => SqlxRenameRule::Camel,
        DbRenameRuleIr::Snake => SqlxRenameRule::Snake,
        DbRenameRuleIr::ScreamingSnake => SqlxRenameRule::ScreamingSnake,
        DbRenameRuleIr::Kebab => SqlxRenameRule::Kebab,
        DbRenameRuleIr::ScreamingKebab => SqlxRenameRule::ScreamingKebab,
    }
}

/// Returns true for database class annotation names.
pub(super) fn is_database_config(name: &str) -> bool {
    matches!(name, DATABASE | SQLX_DATABASE)
}

/// Returns true for DAO class annotation names.
pub(super) fn is_dao_config(name: &str) -> bool {
    matches!(name, DAO | SQLX_DAO)
}

/// Converts SQLx rename rules to the shared serde rename rule enum.
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

/// Parses a database driver or database type enum member from source text.
///
/// `Driver.sqlite3` and `SqlxDatabaseType.sqlite` are two spellings of one
/// dialect, and the dialect owns both.
fn parse_driver(source: &str) -> Option<DbDriver> {
    Dialect::from_annotation(source).map(|dialect| dialect.driver)
}

/// Parses a SQLx rename rule enum member from source text.
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
#[path = "annotations/tests.rs"]
mod tests;
