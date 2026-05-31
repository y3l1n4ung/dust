use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use dust_dart_emit::{
    apply_rename_rule, balanced_parenthesized, parse_bool_literal, parse_named_arguments,
    parse_string_literal, split_top_level_items,
};
use dust_ir::{ConfigApplicationIr, LibraryIr, SerdeRenameRuleIr, SpanIr, SymbolId, TypeIr};
use dust_plugin_api::short_symbol_name;
use dust_text::TextRange;

use super::{
    constants::{DAO, DATABASE, FROM_ROW, FROM_ROW_SYMBOL, QUERY, SQLX, SQLX_DAO, SQLX_DATABASE},
    model::{
        DaoClass, DaoMethod, DatabaseClass, DbDriver, FetchMode, QueryFunction, QuerySpec,
        RowClass, SqlxConfig, SqlxRenameRule,
    },
};

pub(crate) fn config_name(symbol: &SymbolId) -> &str {
    short_symbol_name(&symbol.0)
}

pub(crate) fn has_config(configs: &[ConfigApplicationIr], expected: &str) -> bool {
    configs
        .iter()
        .any(|config| config_name(&config.symbol) == expected)
}

pub(crate) fn database_classes(library: &LibraryIr) -> Vec<DatabaseClass<'_>> {
    library
        .classes
        .iter()
        .filter_map(|class| {
            let config = class
                .configs
                .iter()
                .find(|config| is_database_config(config_name(&config.symbol)))?;
            let parsed = parse_database_config(config)?;
            Some(DatabaseClass {
                class,
                driver: parsed.driver,
                migrations: parsed.migrations,
            })
        })
        .collect()
}

pub(crate) fn dao_classes(library: &LibraryIr) -> Vec<DaoClass<'_>> {
    library
        .classes
        .iter()
        .filter(|class| {
            class
                .configs
                .iter()
                .any(|config| is_dao_config(config_name(&config.symbol)))
        })
        .map(|class| DaoClass {
            class,
            methods: class
                .methods
                .iter()
                .filter_map(|method| {
                    let config = method
                        .configs
                        .iter()
                        .find(|config| config_name(&config.symbol) == QUERY)?;
                    let (sql, sql_source_static) = parse_query_config(config);
                    Some(DaoMethod {
                        method,
                        sql,
                        sql_source_static,
                        return_ok_type: result_ok_type(&method.return_type).cloned(),
                    })
                })
                .collect(),
        })
        .collect()
}

pub(crate) fn row_classes(library: &LibraryIr) -> Vec<RowClass<'_>> {
    library
        .classes
        .iter()
        .filter(|class| {
            class
                .traits
                .iter()
                .any(|item| item.symbol.0 == FROM_ROW_SYMBOL)
                || has_config(&class.configs, FROM_ROW)
        })
        .map(|class| RowClass {
            class,
            config: sqlx_config(&class.configs),
        })
        .collect()
}

pub(crate) fn imported_row_names(library: &LibraryIr) -> HashSet<String> {
    library
        .imports
        .iter()
        .filter_map(|uri| resolve_import_path(library, uri))
        .filter_map(|path| fs::read_to_string(path).ok())
        .flat_map(|source| row_names_from_source(&source))
        .collect()
}

pub(crate) fn query_specs(library: &LibraryIr) -> Vec<QuerySpec> {
    let source_path = Path::new(&library.package_root).join(&library.source_path);
    let Ok(source) = fs::read_to_string(source_path) else {
        return dao_query_specs(library);
    };
    let mut specs = parse_query_specs_from_source(library.span, &source);
    specs.extend(dao_query_specs(library));
    specs.sort_by_key(|spec| spec.span.range.start());
    specs
}

pub(crate) fn dao_query_specs(library: &LibraryIr) -> Vec<QuerySpec> {
    dao_classes(library)
        .into_iter()
        .flat_map(|dao| {
            dao.methods.into_iter().map(move |method| {
                let (function, fetch, row_type, scalar_type) =
                    query_shape_from_return(method.return_ok_type.as_ref());
                QuerySpec {
                    function,
                    fetch,
                    sql: method.sql,
                    sql_source_static: method.sql_source_static,
                    row_type,
                    scalar_type,
                    parameter_count: method.method.params.len(),
                    params_source_is_list: true,
                    span: method.method.span,
                    display_name: Some(format!("{}.{}", dao.class.name, method.method.name)),
                }
            })
        })
        .collect()
}

pub(crate) fn parse_query_specs_from_source(library_span: SpanIr, source: &str) -> Vec<QuerySpec> {
    let mut specs = Vec::new();
    for function in [
        ("queryAs", QueryFunction::As),
        ("queryScalar", QueryFunction::Scalar),
        ("queryRaw", QueryFunction::Raw),
        ("queryExecute", QueryFunction::Execute),
    ] {
        collect_query_function_calls(library_span, source, function.0, function.1, &mut specs);
    }
    specs.sort_by_key(|spec| spec.span.range.start());
    specs
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

fn collect_query_function_calls(
    library_span: SpanIr,
    source: &str,
    name: &str,
    function: QueryFunction,
    out: &mut Vec<QuerySpec>,
) {
    let mut offset = 0;
    while let Some(relative) = source[offset..].find(name) {
        let start = offset + relative;
        if !is_identifier_boundary(source, start, name.len()) {
            offset = start + name.len();
            continue;
        }
        let Some((type_arg, after_type)) = parse_optional_type_arg(source, start + name.len())
        else {
            offset = start + name.len();
            continue;
        };
        let after_type = skip_ws(source, after_type);
        if !source[after_type..].starts_with('(') {
            offset = start + name.len();
            continue;
        }
        let Some(call) = balanced_parenthesized(&source[after_type..]) else {
            offset = start + name.len();
            continue;
        };
        let call_end = after_type + call.len();
        let inner = call
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or("");
        let args = split_top_level_items(inner);
        let (sql, sql_source_static) = args
            .first()
            .and_then(|arg| parse_static_sql_literal(arg).map(|sql| (sql, true)))
            .unwrap_or_else(|| (String::new(), false));
        let params_source = args.get(1).copied().unwrap_or("const <Object?>[]");
        let (params_source_is_list, parameter_count) = parse_list_argument_count(params_source);
        let fetch = parse_fetch_mode(function, &source[call_end..]);
        let (row_type, scalar_type) = match function {
            QueryFunction::As => (type_arg.clone(), None),
            QueryFunction::Scalar => (None, type_arg.as_deref().map(type_from_source)),
            QueryFunction::Raw | QueryFunction::Execute => (None, None),
        };
        out.push(QuerySpec {
            function,
            fetch,
            sql,
            sql_source_static,
            row_type,
            scalar_type,
            parameter_count,
            params_source_is_list,
            span: SpanIr::new(
                library_span.file_id,
                TextRange::new(start as u32, call_end as u32),
            ),
            display_name: None,
        });
        offset = call_end;
    }
}

fn parse_query_config(config: &ConfigApplicationIr) -> (String, bool) {
    let Some(args) = config.arguments_source.as_deref() else {
        return (String::new(), false);
    };
    let inner = args
        .trim()
        .strip_prefix('(')
        .and_then(|source| source.strip_suffix(')'))
        .unwrap_or(args)
        .trim();
    if let Some(sql) = parse_static_sql_literal(inner) {
        return (sql, true);
    }
    for item in split_top_level_items(inner) {
        let Some((key, value)) = item.split_once(':') else {
            continue;
        };
        if key.trim() == "sql" {
            return parse_static_sql_literal(value)
                .map(|sql| (sql, true))
                .unwrap_or_else(|| (String::new(), false));
        }
    }
    (String::new(), false)
}

fn query_shape_from_return(
    ok_type: Option<&TypeIr>,
) -> (QueryFunction, FetchMode, Option<String>, Option<TypeIr>) {
    let Some(ok_type) = ok_type else {
        return (QueryFunction::Raw, FetchMode::Raw, None, None);
    };
    if ok_type.is_named("ExecResult") || ok_type.is_named("Unit") {
        return (QueryFunction::Execute, FetchMode::Execute, None, None);
    }
    if ok_type.is_named("List") {
        let Some(item) = ok_type.args().first() else {
            return (QueryFunction::Raw, FetchMode::Raw, None, None);
        };
        if item.is_named("Row") {
            return (QueryFunction::Raw, FetchMode::Raw, None, None);
        }
        return (
            QueryFunction::As,
            FetchMode::All,
            item.name().map(str::to_owned),
            None,
        );
    }
    if is_scalar_type(ok_type) {
        return (
            QueryFunction::Scalar,
            if ok_type.is_nullable() {
                FetchMode::Optional
            } else {
                FetchMode::One
            },
            None,
            Some(ok_type.clone()),
        );
    }
    (
        QueryFunction::As,
        if ok_type.is_nullable() {
            FetchMode::Optional
        } else {
            FetchMode::One
        },
        ok_type.name().map(str::to_owned),
        None,
    )
}

pub(crate) fn result_ok_type(return_type: &TypeIr) -> Option<&TypeIr> {
    let future = return_type
        .is_named("Future")
        .then(|| return_type.args().first())
        .flatten()?;
    let result = future.is_named("Result").then_some(future)?;
    result.args().first()
}

fn is_scalar_type(ty: &TypeIr) -> bool {
    matches!(
        ty.name(),
        Some("String" | "int" | "double" | "num" | "bool" | "DateTime")
    )
}

fn parse_fetch_mode(function: QueryFunction, after_call: &str) -> FetchMode {
    let after = after_call.trim_start();
    if after.starts_with(".fetchOptional") {
        return FetchMode::Optional;
    }
    if after.starts_with(".fetchOne") {
        return FetchMode::One;
    }
    if after.starts_with(".fetchAll") {
        return FetchMode::All;
    }
    if after.starts_with(".fetch") {
        return FetchMode::Raw;
    }
    if after.starts_with(".execute") {
        return FetchMode::Execute;
    }
    match function {
        QueryFunction::Execute => FetchMode::Execute,
        QueryFunction::Raw => FetchMode::Raw,
        _ => FetchMode::One,
    }
}

fn parse_optional_type_arg(source: &str, start: usize) -> Option<(Option<String>, usize)> {
    let start = skip_ws(source, start);
    if !source[start..].starts_with('<') {
        return Some((None, start));
    }
    let mut depth = 0_i32;
    for (relative, ch) in source[start..].char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    let end = start + relative;
                    return Some((Some(source[start + 1..end].trim().to_owned()), end + 1));
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_list_argument_count(source: &str) -> (bool, usize) {
    let mut source = source.trim();
    source = source.strip_prefix("const ").unwrap_or(source).trim();
    if source.starts_with('<') {
        let Some((_, after_type)) = parse_optional_type_arg(source, 0) else {
            return (false, 0);
        };
        source = source[after_type..].trim();
    }
    let Some(inner) = source.strip_prefix('[').and_then(|s| s.strip_suffix(']')) else {
        return (false, 0);
    };
    if inner.trim().is_empty() {
        return (true, 0);
    }
    (true, split_top_level_items(inner).len())
}

fn parse_static_sql_literal(source: &str) -> Option<String> {
    let source = source.trim();
    let (raw, source) = match source.as_bytes() {
        [b'r' | b'R', b'\'' | b'"', ..] => (true, &source[1..]),
        _ => (false, source),
    };
    let quote = source.chars().next()?;
    if !matches!(quote, '\'' | '"') {
        return None;
    }
    let delimiter = if source.starts_with(&quote.to_string().repeat(3)) {
        quote.to_string().repeat(3)
    } else {
        quote.to_string()
    };
    let body_start = delimiter.len();

    let mut sql = String::new();
    let mut escaped = false;
    let mut end_offset = None;
    for (index, ch) in source[body_start..].char_indices() {
        let absolute = body_start + index;
        if !raw && escaped {
            sql.push(ch);
            escaped = false;
            continue;
        }
        if !raw && ch == '\\' {
            escaped = true;
            continue;
        }
        if !raw && ch == '$' {
            return None;
        }
        if source[absolute..].starts_with(&delimiter) {
            end_offset = Some(absolute + delimiter.len());
            break;
        }
        sql.push(ch);
    }

    let end_offset = end_offset?;
    source[end_offset..].trim().is_empty().then_some(sql)
}

fn type_from_source(source: &str) -> TypeIr {
    match source.trim().trim_end_matches('?') {
        "String" => TypeIr::string(),
        "int" => TypeIr::int(),
        "bool" => TypeIr::bool(),
        "double" => TypeIr::double(),
        "num" => TypeIr::num(),
        "Object" => TypeIr::object(),
        other => TypeIr::named(other),
    }
}

fn is_identifier_boundary(source: &str, start: usize, len: usize) -> bool {
    let before = source[..start].chars().next_back();
    let after = source[start + len..].chars().next();
    !before.is_some_and(is_identifier_char) && !after.is_some_and(is_identifier_char)
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}

fn skip_ws(source: &str, mut offset: usize) -> usize {
    while let Some(ch) = source[offset..].chars().next() {
        if !ch.is_whitespace() {
            break;
        }
        offset += ch.len_utf8();
    }
    offset
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

struct DatabaseConfig {
    driver: DbDriver,
    migrations: String,
}

fn parse_database_config(config: &ConfigApplicationIr) -> Option<DatabaseConfig> {
    let mut driver = DbDriver::Sqlite3;
    let mut migrations = "./migrations".to_owned();
    for (key, value) in parse_named_arguments(config.arguments_source.as_deref()) {
        match key {
            "driver" => {
                if let Some(parsed) = parse_driver(value) {
                    driver = parsed;
                }
            }
            "type" => {
                if let Some(parsed) = parse_database_type(value) {
                    driver = parsed;
                }
            }
            "migrations" => {
                if let Some(parsed) = parse_string_literal(value) {
                    migrations = parsed;
                }
            }
            _ => {}
        }
    }
    Some(DatabaseConfig { driver, migrations })
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

fn is_database_config(name: &str) -> bool {
    matches!(name, DATABASE | SQLX_DATABASE)
}

fn is_dao_config(name: &str) -> bool {
    matches!(name, DAO | SQLX_DAO)
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

fn resolve_import_path(library: &LibraryIr, uri: &str) -> Option<PathBuf> {
    if uri.starts_with("dart:") || uri.starts_with("package:flutter/") {
        return None;
    }
    if let Some(rest) = uri.strip_prefix("package:") {
        let (package, path) = rest.split_once('/')?;
        if package == library.package_name {
            return Some(Path::new(&library.package_root).join("lib").join(path));
        }
        return None;
    }
    let source_dir = Path::new(&library.package_root)
        .join(&library.source_path)
        .parent()?
        .to_path_buf();
    Some(normalize_path(&source_dir.join(uri)))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn row_names_from_source(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut metadata = String::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('@') {
            metadata.push_str(trimmed);
            continue;
        }
        if let Some(name) = class_name_from_line(trimmed) {
            if metadata.contains("FromRow") {
                names.push(name.to_owned());
            }
            metadata.clear();
            continue;
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            metadata.clear();
        }
    }
    names
}

fn class_name_from_line(line: &str) -> Option<&str> {
    let rest = line
        .strip_prefix("class ")
        .or_else(|| line.strip_prefix("final class "))
        .or_else(|| line.strip_prefix("abstract class "))
        .or_else(|| line.strip_prefix("abstract final class "))?;
    rest.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .next()
        .filter(|name| !name.is_empty())
}

#[cfg(test)]
mod tests {
    use dust_text::{FileId, TextRange};

    use super::*;

    fn span() -> SpanIr {
        SpanIr::new(FileId::new(1), TextRange::new(0_u32, 1_u32))
    }

    #[test]
    fn parses_query_calls_from_source() {
        let specs = parse_query_specs_from_source(
            span(),
            "Future<User?> find(Pool db, int id) => queryAs<UserRow>(r'SELECT * FROM users WHERE id = $1', [id]).fetchOptional(db);",
        );

        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].function, QueryFunction::As);
        assert_eq!(specs[0].fetch, FetchMode::Optional);
        assert_eq!(specs[0].row_type.as_deref(), Some("UserRow"));
        assert_eq!(specs[0].sql, "SELECT * FROM users WHERE id = $1");
        assert_eq!(specs[0].parameter_count, 1);
        assert!(specs[0].params_source_is_list);
    }

    #[test]
    fn rejects_non_list_query_params_during_parse() {
        let specs = parse_query_specs_from_source(
            span(),
            "Future<int> count(Pool db, List<Object?> args) => queryScalar<int>('SELECT COUNT(*) FROM users', args).fetchOne(db);",
        );

        assert_eq!(specs.len(), 1);
        assert!(!specs[0].params_source_is_list);
    }

    #[test]
    fn accepts_raw_static_sql_literal_with_placeholders() {
        let specs = parse_query_specs_from_source(
            span(),
            "Future<User?> find(Pool db, int id) => queryAs<UserRow>(r'SELECT * FROM users WHERE id = $1', [id]).fetchOptional(db);",
        );

        assert_eq!(specs.len(), 1);
        assert!(specs[0].sql_source_static);
        assert_eq!(specs[0].sql, "SELECT * FROM users WHERE id = $1");
    }

    #[test]
    fn accepts_raw_multiline_static_sql_literal_with_placeholders() {
        let specs = parse_query_specs_from_source(
            span(),
            "Future<List<Row>> all(Pool db, int id) => queryRaw(r'''\nSELECT *\nFROM users\nWHERE id = $1\n''', [id]).fetch(db);",
        );

        assert_eq!(specs.len(), 1);
        assert!(specs[0].sql_source_static);
        assert_eq!(specs[0].sql, "\nSELECT *\nFROM users\nWHERE id = $1\n");
        assert_eq!(specs[0].parameter_count, 1);
    }

    #[test]
    fn rejects_interpolated_sql_literal() {
        let specs = parse_query_specs_from_source(
            span(),
            "Future<List<Row>> all(Pool db, String table) => queryRaw('SELECT * FROM $table', []).fetch(db);",
        );

        assert_eq!(specs.len(), 1);
        assert!(!specs[0].sql_source_static);
    }

    #[test]
    fn rejects_concatenated_sql_literals() {
        let specs = parse_query_specs_from_source(
            span(),
            "Future<List<Row>> all(Pool db) => queryRaw('SELECT * ' 'FROM users', []).fetch(db);",
        );

        assert_eq!(specs.len(), 1);
        assert!(!specs[0].sql_source_static);
    }

    #[test]
    fn rejects_sql_variable() {
        let specs = parse_query_specs_from_source(
            span(),
            "Future<List<Row>> all(Pool db, String sql) => queryRaw(sql, []).fetch(db);",
        );

        assert_eq!(specs.len(), 1);
        assert!(!specs[0].sql_source_static);
    }

    #[test]
    fn rejects_const_sql_variable() {
        let specs = parse_query_specs_from_source(
            span(),
            "Future<List<Row>> all(Pool db) { const sql = 'SELECT * FROM users'; return queryRaw(sql, []).fetch(db); }",
        );

        assert_eq!(specs.len(), 1);
        assert!(!specs[0].sql_source_static);
    }
}
