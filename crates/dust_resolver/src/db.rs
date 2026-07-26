use dust_ir::{
    AnnotationValueIr, ConfigApplicationIr, DbConfigIr, DbDatabaseConfigIr, DbDriverIr,
    DbQueryConfigIr, DbRenameRuleIr, DbSqlxConfigIr, NormalizedConfigIr,
};

/// Normalizes database, SQLx, DAO, row, and query annotations.
pub(crate) fn normalize_db(
    configs: &mut [ConfigApplicationIr],
    fields: &mut [crate::ResolvedField],
    constructors: &mut [crate::ResolvedConstructor],
    methods: &mut [crate::ResolvedMethod],
) {
    for config in configs {
        let snapshot = config.clone();
        let Some(normalized) = normalize_config(&snapshot) else {
            continue;
        };
        config.normalized = Some(NormalizedConfigIr::Db(normalized));
    }
    for field in fields {
        normalize_db(&mut field.configs, &mut [], &mut [], &mut []);
    }
    for constructor in constructors {
        normalize_db(&mut constructor.configs, &mut [], &mut [], &mut []);
    }
    for method in methods {
        normalize_db(&mut method.configs, &mut [], &mut [], &mut []);
        for param in &mut method.params {
            normalize_db(&mut param.configs, &mut [], &mut [], &mut []);
        }
    }
}

/// Converts one resolved database annotation into typed configuration.
fn normalize_config(config: &ConfigApplicationIr) -> Option<DbConfigIr> {
    match config.symbol.0.as_str() {
        "dust_dart::Database" | "dust_dart::SqlxDatabase" => {
            let driver = config
                .named_member("driver")
                .or_else(|| config.named_member("type"))
                .and_then(|value| match value.rsplit('.').next()? {
                    "sqlite" | "sqlite3" => Some(DbDriverIr::Sqlite3),
                    "postgres" => Some(DbDriverIr::Postgres),
                    _ => None,
                })
                .unwrap_or(DbDriverIr::Sqlite3);
            Some(DbConfigIr::Database(DbDatabaseConfigIr {
                driver,
                migrations: config
                    .named_string("migrations")
                    .unwrap_or_else(|| "./migrations".to_owned()),
            }))
        }
        "dust_dart::Sqlx" => Some(DbConfigIr::Sqlx(sqlx_config(config))),
        "dust_dart::Query" => {
            let sql = config
                .positional_string(0)
                .or_else(|| config.named_string("sql"))
                .unwrap_or_default();
            Some(DbConfigIr::Query(DbQueryConfigIr {
                sql,
                sql_source_static: config.positional_string(0).is_some()
                    || config.named_string("sql").is_some(),
            }))
        }
        "dust_dart::Dao" | "dust_dart::SqlxDao" | "dust_dart::FromRow" => Some(DbConfigIr::Marker),
        _ => None,
    }
}

/// Converts one resolved SQLx annotation into typed configuration.
fn sqlx_config(config: &ConfigApplicationIr) -> DbSqlxConfigIr {
    let try_from_source = config
        .named_argument_source("tryFrom")
        .map(str::trim)
        .map(str::to_owned);
    let try_from_class_name = try_from_class_name(config);

    let rename_all =
        config
            .named_member("renameAll")
            .and_then(|value| match value.rsplit('.').next()? {
                "lowerCase" => Some(DbRenameRuleIr::Lower),
                "upperCase" => Some(DbRenameRuleIr::Upper),
                "pascalCase" => Some(DbRenameRuleIr::Pascal),
                "camelCase" => Some(DbRenameRuleIr::Camel),
                "snakeCase" => Some(DbRenameRuleIr::Snake),
                "screamingSnakeCase" => Some(DbRenameRuleIr::ScreamingSnake),
                "kebabCase" => Some(DbRenameRuleIr::Kebab),
                "screamingKebabCase" => Some(DbRenameRuleIr::ScreamingKebab),
                _ => None,
            });
    DbSqlxConfigIr {
        rename: config.named_string("rename"),
        rename_all,
        flatten: config.named_bool("flatten").unwrap_or(false),
        default_value_source: config
            .named_argument_source("defaultValue")
            .map(str::trim)
            .map(str::to_owned),
        skip: config.named_bool("skip").unwrap_or(false),
        json: config.named_bool("json").unwrap_or(false),
        try_from_source,
        try_from_class_name,
    }
}

/// Returns the converter class referenced by `Sqlx(tryFrom: ...)`.
fn try_from_class_name(config: &ConfigApplicationIr) -> Option<String> {
    match config.named_argument_value("tryFrom") {
        Some(AnnotationValueIr::Constructor { name, .. })
        | Some(AnnotationValueIr::Member(name)) => Some(name.short.clone()),
        _ => config
            .named_argument_source("tryFrom")
            .and_then(try_from_converter_name)
            .map(str::to_owned),
    }
}

/// Extracts a converter class name from a preserved `tryFrom` expression.
fn try_from_converter_name(source: &str) -> Option<&str> {
    let value = source.trim();
    let value = value.strip_prefix("const ").unwrap_or(value).trim();
    let before_args = value.split_once('(').map_or(value, |(name, _)| name).trim();
    before_args
        .rsplit('.')
        .next()
        .filter(|name| !name.is_empty())
}
