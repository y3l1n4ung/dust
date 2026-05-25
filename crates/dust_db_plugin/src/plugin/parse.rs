use dust_dart_emit::{
    apply_rename_rule, balanced_parenthesized, normalized_args, parse_bool_literal,
    parse_named_arguments, parse_string_literal, split_top_level_items,
};
use dust_ir::{ConfigApplicationIr, MethodIr, SerdeRenameRuleIr, SymbolId};
use dust_plugin_api::short_symbol_name;

use super::{
    constants::{DUST_DB, FROM_ROW, QUERY, SQLX, TRANSACTION},
    model::{DbClass, FetchKind, QuerySpec, RowClass, SqlxConfig, SqlxRenameRule},
};

pub(crate) fn config_name(symbol: &SymbolId) -> &str {
    short_symbol_name(&symbol.0)
}

pub(crate) fn has_config(configs: &[ConfigApplicationIr], expected: &str) -> bool {
    configs
        .iter()
        .any(|config| config_name(&config.symbol) == expected)
}

pub(crate) fn dust_db_classes(library: &dust_ir::LibraryIr) -> Vec<DbClass<'_>> {
    library
        .classes
        .iter()
        .filter_map(|class| {
            let config = class
                .configs
                .iter()
                .find(|config| config_name(&config.symbol) == DUST_DB)?;
            let migrations = parse_dust_db_config(config)?.migrations;
            let queries = class
                .methods
                .iter()
                .filter_map(parse_query_method)
                .collect::<Vec<_>>();
            Some(DbClass {
                class,
                migrations,
                queries,
            })
        })
        .collect()
}

pub(crate) fn row_classes(library: &dust_ir::LibraryIr) -> Vec<RowClass<'_>> {
    library
        .classes
        .iter()
        .filter(|class| has_config(&class.configs, FROM_ROW))
        .map(|class| RowClass {
            class,
            config: sqlx_config(&class.configs),
        })
        .collect()
}

pub(crate) fn parse_query_method(method: &MethodIr) -> Option<QuerySpec<'_>> {
    let query_config = method
        .configs
        .iter()
        .find(|config| config_name(&config.symbol) == QUERY);
    let sql_from_config =
        query_config.and_then(|config| parse_query_config(config.arguments_source.as_deref()));
    let body = method.body_source.as_deref();
    let sql_from_body = body.and_then(parse_body_query_sql);
    let sql = sql_from_config.or(sql_from_body)?;
    let fetch = body
        .and_then(parse_fetch_kind)
        .unwrap_or_else(|| infer_fetch_kind(method, &sql));
    let args = body
        .and_then(parse_fetch_args)
        .filter(|args| !args.is_empty())
        .unwrap_or_else(|| {
            method
                .params
                .iter()
                .map(|param| param.name.clone())
                .collect()
        });
    let transaction = has_config(&method.configs, TRANSACTION);

    Some(QuerySpec {
        method,
        sql,
        fetch,
        args,
        transaction,
    })
}

pub(crate) fn sqlx_config(configs: &[ConfigApplicationIr]) -> SqlxConfig {
    let mut out = SqlxConfig::default();
    for config in configs {
        if config_name(&config.symbol) != SQLX {
            continue;
        }
        for (key, value) in parse_named_arguments(config.arguments_source.as_deref()) {
            match key {
                "rename" => out.rename = parse_string_literal(value),
                "renameAll" => out.rename_all = parse_rename_rule(value),
                "flatten" => out.flatten = parse_bool_literal(value).unwrap_or(out.flatten),
                "defaultValue" => out.default_value_source = Some(value.trim().to_owned()),
                "skip" => out.skip = parse_bool_literal(value).unwrap_or(out.skip),
                "json" => out.json = parse_bool_literal(value).unwrap_or(out.json),
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

struct DustDbConfig {
    migrations: String,
}

fn parse_dust_db_config(config: &ConfigApplicationIr) -> Option<DustDbConfig> {
    let migrations = parse_named_arguments(config.arguments_source.as_deref())
        .into_iter()
        .find_map(|(key, value)| (key == "migrations").then(|| parse_string_literal(value))?)?;
    Some(DustDbConfig { migrations })
}

fn parse_query_config(args: Option<&str>) -> Option<String> {
    let inner = normalized_args(args?)?;
    let first = split_top_level_items(inner).into_iter().next()?;
    parse_string_literal(first)
}

fn parse_body_query_sql(body: &str) -> Option<String> {
    let start = body.find("@Query")?;
    let args = annotation_args(&body[start + "@Query".len()..])?;
    parse_query_config(Some(args))
}

fn parse_fetch_kind(body: &str) -> Option<FetchKind> {
    let start = body.find("$fetch.")? + "$fetch.".len();
    let method = body[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect::<String>();
    match method.as_str() {
        "one" => Some(FetchKind::One),
        "all" => Some(FetchKind::All),
        "scalar" => Some(FetchKind::Scalar),
        "insertOne" => Some(FetchKind::InsertOne),
        "execute" => Some(FetchKind::Execute),
        "stream" => Some(FetchKind::Stream),
        _ => None,
    }
}

fn parse_fetch_args(body: &str) -> Option<Vec<String>> {
    let start = body.find("$fetch.")? + "$fetch.".len();
    let after_name = body[start..].find('(').map(|index| start + index)?;
    let args = balanced_parenthesized(&body[after_name..])?;
    let inner = args.strip_prefix('(')?.strip_suffix(')')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    let inner = inner
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(inner)
        .trim();
    Some(
        split_top_level_items(inner)
            .into_iter()
            .map(str::to_owned)
            .collect(),
    )
}

fn infer_fetch_kind(method: &MethodIr, sql: &str) -> FetchKind {
    if method.return_type.is_named("Stream") {
        return FetchKind::Stream;
    }
    if !method.return_type.is_named("Future") || method.return_type.args().len() != 1 {
        return FetchKind::One;
    }

    let inner = &method.return_type.args()[0];
    if inner.is_named("List") {
        return FetchKind::All;
    }
    if inner.is_named("void") || inner.name() == Some("void") {
        return FetchKind::Execute;
    }
    if sql
        .split_whitespace()
        .next()
        .is_some_and(|keyword| keyword.eq_ignore_ascii_case("insert"))
    {
        return FetchKind::InsertOne;
    }
    if matches!(
        inner.name(),
        Some("int" | "double" | "num" | "String" | "bool")
    ) {
        return FetchKind::Scalar;
    }

    FetchKind::One
}

fn annotation_args(source: &str) -> Option<&str> {
    let start = source.find('(')?;
    balanced_parenthesized(&source[start..])
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
    use dust_ir::{MethodIr, SpanIr, TypeIr};
    use dust_text::{FileId, TextRange};

    use super::{infer_fetch_kind, parse_body_query_sql, parse_fetch_args, parse_fetch_kind};
    use crate::plugin::model::FetchKind;

    fn method(return_type: TypeIr) -> MethodIr {
        MethodIr {
            name: "query".to_owned(),
            is_static: false,
            is_external: false,
            return_type,
            has_body: false,
            body_source: None,
            span: SpanIr::new(FileId::new(1), TextRange::new(0_u32, 1_u32)),
            params: Vec::new(),
            traits: Vec::new(),
            configs: Vec::new(),
        }
    }

    #[test]
    fn parses_query_carrier_body() {
        let body = "=> @Query('SELECT * FROM users WHERE id = ?') $fetch.one(id);";
        assert_eq!(
            parse_body_query_sql(body).as_deref(),
            Some("SELECT * FROM users WHERE id = ?")
        );
        assert_eq!(parse_fetch_kind(body), Some(FetchKind::One));
        assert_eq!(parse_fetch_args(body), Some(vec!["id".to_owned()]));
    }

    #[test]
    fn infers_fetch_kind_from_plain_query_signature() {
        assert_eq!(
            infer_fetch_kind(
                &method(TypeIr::generic(
                    "Future",
                    vec![TypeIr::generic("List", vec![TypeIr::named("User")])]
                )),
                "SELECT * FROM users"
            ),
            FetchKind::All
        );
        assert_eq!(
            infer_fetch_kind(
                &method(TypeIr::generic("Future", vec![TypeIr::int()])),
                "SELECT COUNT(*) FROM users"
            ),
            FetchKind::Scalar
        );
        assert_eq!(
            infer_fetch_kind(
                &method(TypeIr::generic("Future", vec![TypeIr::named("User")])),
                "INSERT INTO users(name) VALUES (?)"
            ),
            FetchKind::InsertOne
        );
        assert_eq!(
            infer_fetch_kind(
                &method(TypeIr::generic("Stream", vec![TypeIr::named("User")])),
                "SELECT * FROM users"
            ),
            FetchKind::Stream
        );
    }
}
