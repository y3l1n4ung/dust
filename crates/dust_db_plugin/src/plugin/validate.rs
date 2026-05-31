use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use dust_dart_emit::DYNAMIC_TYPES;
use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_ir::{ClassIr, FieldIr, LibraryIr, TypeIr};
use either::Either;
use serde::{Deserialize, Serialize};
use sqlx::{Column, Connection, Executor, sqlite::SqliteConnection};

use super::{
    DbPluginOptions,
    model::{DatabaseClass, DbDriver, FetchMode, QueryFunction, QuerySpec, RowClass, SqlxConfig},
    parse::{
        dao_classes, database_classes, effective_column_name, imported_row_names, query_specs,
        result_ok_type, row_classes, sqlx_config,
    },
};

pub(crate) fn validate_db_library(
    library: &LibraryIr,
    options: DbPluginOptions,
) -> Vec<Diagnostic> {
    if !options.databases && !database_classes(library).is_empty() {
        return Vec::new();
    }

    let rows = row_classes(library);
    let mut diagnostics = Vec::new();
    validate_rows(&rows, &mut diagnostics);
    if options.databases {
        validate_databases(library, options, &rows, &mut diagnostics);
    }
    diagnostics
}

fn validate_rows(rows: &[RowClass<'_>], diagnostics: &mut Vec<Diagnostic>) {
    let row_by_name = rows
        .iter()
        .map(|row| (row.class.name.as_str(), row))
        .collect::<HashMap<_, _>>();
    let row_names = row_by_name.keys().copied().collect::<HashSet<_>>();
    for row in rows {
        let mut seen = HashMap::<String, &FieldIr>::new();
        for field in &row.class.fields {
            let config = sqlx_config(&field.configs);
            validate_field_shape(row.class, field, &config, &row_names, diagnostics);
            if config.skip {
                continue;
            }
            if config.flatten {
                if let Some(flattened) = field.ty.name().and_then(|name| row_by_name.get(name)) {
                    for column in collect_row_columns(flattened, &row_by_name) {
                        if let Some(existing) = seen.insert(column.clone(), field) {
                            push_duplicate_column(row.class, field, existing, &column, diagnostics);
                        }
                    }
                }
                continue;
            }
            let column = effective_column_name(&row.config, &field.name, &config);
            if let Some(existing) = seen.insert(column.clone(), field) {
                push_duplicate_column(row.class, field, existing, &column, diagnostics);
            }
        }
    }
}

fn validate_databases(
    library: &LibraryIr,
    options: DbPluginOptions,
    rows: &[RowClass<'_>],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let databases = database_classes(library);
    if databases.is_empty() {
        return;
    }
    let row_columns = row_column_map(rows);
    let queries = query_specs(library);
    for db in &databases {
        validate_database_config(db, diagnostics);
    }
    validate_daos(library, rows, diagnostics);
    for query in &queries {
        validate_query_shape(query, &row_columns, diagnostics);
    }
    if let Some(db) = databases.first() {
        validate_sqlx_describe(library, db, &queries, &row_columns, options, diagnostics);
    }
}

fn validate_daos(library: &LibraryIr, rows: &[RowClass<'_>], diagnostics: &mut Vec<Diagnostic>) {
    let imported_rows = imported_row_names(library);
    let row_names = rows
        .iter()
        .map(|row| row.class.name.as_str())
        .chain(imported_rows.iter().map(String::as_str))
        .collect::<HashSet<_>>();
    for dao in dao_classes(library) {
        let expected_target = format!("_${}", dao.class.name);
        let has_redirecting_factory = dao.class.constructors.iter().any(|constructor| {
            constructor.is_factory
                && constructor.redirected_target_name.as_deref() == Some(expected_target.as_str())
                && constructor.params.len() == 1
                && constructor.params[0].ty.is_named("SqlxDriver")
                && matches!(constructor.params[0].kind, dust_ir::ParamKind::Positional)
        });
        if !has_redirecting_factory {
            diagnostics.push(
                Diagnostic::error(format!(
                    "SqlxDao `{}` must declare `const factory {}(SqlxDriver db) = _${}`",
                    dao.class.name, dao.class.name, dao.class.name
                ))
                .with_label(SourceLabel::new(
                    dao.class.span.file_id,
                    dao.class.span.range,
                    "invalid SQLx DAO constructor",
                )),
            );
        }
        for method in dao.methods {
            validate_dao_method(&dao.class.name, &method, &row_names, diagnostics);
        }
    }
}

fn validate_dao_method(
    dao_name: &str,
    method: &super::model::DaoMethod<'_>,
    row_names: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if method.method.has_body {
        diagnostics.push(
            Diagnostic::error(format!(
                "SqlxDao query method `{}.{}` must be abstract",
                dao_name, method.method.name
            ))
            .with_label(SourceLabel::new(
                method.method.span.file_id,
                method.method.span.range,
                "remove the method body from generated DAO queries",
            )),
        );
    }
    for param in &method.method.params {
        if !matches!(param.kind, dust_ir::ParamKind::Positional) || param.has_default {
            diagnostics.push(
                Diagnostic::error(format!(
                    "SqlxDao query method `{}.{}` only supports required positional parameters in v1",
                    dao_name, method.method.name
                ))
                .with_label(SourceLabel::new(
                    param.span.file_id,
                    param.span.range,
                    "unsupported DAO parameter",
                )),
            );
        }
    }
    if !method.sql_source_static {
        diagnostics.push(
            Diagnostic::error(format!(
                "SqlxDao query method `{}.{}` SQL must be a static raw string literal",
                dao_name, method.method.name
            ))
            .with_label(SourceLabel::new(
                method.method.span.file_id,
                method.method.span.range,
                "invalid @Query SQL",
            )),
        );
    }
    let Some(ok_type) = result_ok_type(&method.method.return_type) else {
        diagnostics.push(
            Diagnostic::error(format!(
                "SqlxDao query method `{}.{}` must return Future<Result<T, SqlxError>>",
                dao_name, method.method.name
            ))
            .with_label(SourceLabel::new(
                method.method.span.file_id,
                method.method.span.range,
                "invalid DAO return type",
            )),
        );
        return;
    };
    if !is_supported_dao_ok_type(ok_type, row_names) {
        diagnostics.push(
            Diagnostic::error(format!(
                "unsupported SqlxDao result type `{}` on `{}.{}`",
                DYNAMIC_TYPES.render(ok_type),
                dao_name,
                method.method.name
            ))
            .with_label(SourceLabel::new(
                method.method.span.file_id,
                method.method.span.range,
                "unsupported DAO result type",
            )),
        );
    }
}

fn is_supported_dao_ok_type(ty: &TypeIr, row_names: &HashSet<&str>) -> bool {
    if ty.is_named("ExecResult") || ty.is_named("Unit") || is_supported_scalar_type(ty) {
        return true;
    }
    if ty.is_named("List") {
        let Some(item) = ty.args().first() else {
            return false;
        };
        return item.is_named("Row") || item.name().is_some_and(|name| row_names.contains(name));
    }
    ty.name().is_some_and(|name| row_names.contains(name))
}

fn validate_database_config(db: &DatabaseClass<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if db.migrations.trim().is_empty() {
        diagnostics.push(Diagnostic::error(format!(
            "Database class `{}` must provide a migrations path",
            db.class.name
        )));
    }
    if matches!(db.driver, DbDriver::Postgres) {
        diagnostics.push(
            Diagnostic::error("Driver.postgres is reserved for a future Dust DB release")
                .with_label(SourceLabel::new(
                    db.class.span.file_id,
                    db.class.span.range,
                    "use Driver.sqlite3 in v1",
                )),
        );
    }
}

fn validate_query_shape(
    query: &QuerySpec,
    _row_columns: &HashMap<String, HashSet<String>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !query.sql_source_static {
        diagnostics.push(query_error(
            query,
            "Dust DB query SQL must be a static string literal",
        ));
        return;
    }
    if !query.params_source_is_list {
        diagnostics.push(query_error(
            query,
            "Dust DB query parameters must be a List literal in v1",
        ));
    }
    match validate_placeholders(&query.sql, query.parameter_count) {
        Ok(_) => {}
        Err(error) => diagnostics.push(query_error(query, error)),
    }
    match query.function {
        QueryFunction::As => {
            if query.row_type.is_none() {
                diagnostics.push(query_error(query, "queryAs<T> must specify a row type"));
                return;
            }
            if !matches!(
                query.fetch,
                FetchMode::One | FetchMode::Optional | FetchMode::All
            ) {
                diagnostics.push(query_error(
                    query,
                    "queryAs<T> must end with fetchOne, fetchOptional, or fetchAll",
                ));
            }
        }
        QueryFunction::Scalar => {
            if query
                .scalar_type
                .as_ref()
                .is_none_or(|ty| !is_supported_scalar_type(ty))
            {
                diagnostics.push(query_error(
                    query,
                    "queryScalar<T> must use a supported scalar type",
                ));
            }
            if !matches!(query.fetch, FetchMode::One | FetchMode::Optional) {
                diagnostics.push(query_error(
                    query,
                    "queryScalar<T> must end with fetchOne or fetchOptional",
                ));
            }
        }
        QueryFunction::Raw if query.fetch != FetchMode::Raw => {
            diagnostics.push(query_error(query, "queryRaw must end with fetch"))
        }
        QueryFunction::Execute if query.fetch != FetchMode::Execute => {
            diagnostics.push(query_error(query, "queryExecute must end with execute"))
        }
        QueryFunction::Raw | QueryFunction::Execute => {}
    }
}

fn validate_sqlx_describe(
    library: &LibraryIr,
    db: &DatabaseClass<'_>,
    queries: &[QuerySpec],
    row_columns: &HashMap<String, HashSet<String>>,
    options: DbPluginOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if queries.is_empty() || !matches!(db.driver, DbDriver::Sqlite3) {
        return;
    }
    let migrations_path = Path::new(&library.package_root).join(&db.migrations);
    if !migrations_path.exists() {
        diagnostics.push(Diagnostic::error(format!(
            "Database migrations path `{}` does not exist",
            migrations_path.display()
        )));
        return;
    }

    let schema_hash = match schema_hash(&migrations_path) {
        Ok(hash) => hash,
        Err(error) => {
            diagnostics.push(Diagnostic::error(error));
            return;
        }
    };

    if options.offline {
        match validate_from_query_cache(library, &db.migrations, &schema_hash, queries, row_columns)
        {
            Ok(()) => {}
            Err(error) => diagnostics.push(Diagnostic::error(error)),
        }
        return;
    }

    match run_sqlx_validation(
        &migrations_path,
        &db.migrations,
        &schema_hash,
        queries,
        row_columns,
    ) {
        Ok(metadata) => {
            if options.write_metadata {
                if let Err(error) = write_query_cache(library, metadata) {
                    diagnostics.push(Diagnostic::warning(error));
                }
            }
        }
        Err(error) => diagnostics.push(Diagnostic::error(error)),
    }
}

fn run_sqlx_validation(
    migrations_path: &Path,
    migrations: &str,
    schema_hash: &str,
    queries: &[QuerySpec],
    row_columns: &HashMap<String, HashSet<String>>,
) -> Result<Vec<QueryCacheEntry>, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("failed to create SQL validation runtime: {error}"))?;
    runtime.block_on(async move {
        let database_url =
            std::env::var("DUST_DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_owned());
        let mut conn = SqliteConnection::connect(&database_url)
            .await
            .map_err(|error| {
                format!("failed to connect SQL validation database `{database_url}`: {error}")
            })?;
        for migration in migration_files(migrations_path)? {
            let sql = fs::read_to_string(&migration).map_err(|error| {
                format!(
                    "failed to read migration `{}`: {error}",
                    migration.display()
                )
            })?;
            conn.execute(sql.as_str()).await.map_err(|error| {
                format!(
                    "failed to apply migration `{}`: {error}",
                    migration.display()
                )
            })?;
        }
        let mut metadata = Vec::new();
        for query in queries {
            if !query.sql_source_static {
                continue;
            }
            let rewrite = validate_placeholders(&query.sql, query.parameter_count)?;
            let describe = conn
                .describe(rewrite.sql.as_str())
                .await
                .map_err(|error| format!("SQLx rejected `{}`: {error}", query.display_name()))?;
            let parameter_count = describe.parameters().map_or(0, |params| match params {
                Either::Left(values) => values.len(),
                Either::Right(count) => count,
            });
            if parameter_count != rewrite.expanded_parameter_count {
                return Err(format!(
                    "SQLx query `{}` expects {parameter_count} expanded parameters but Dust rewrote {} placeholders",
                    query.display_name(),
                    rewrite.expanded_parameter_count
                ));
            }
            validate_described_columns(query, row_columns, &describe)?;
            metadata.push(QueryCacheEntry {
                migrations: migrations.to_owned(),
                schema_hash: schema_hash.to_owned(),
                sql_hash: stable_hash_hex(query.sql.as_bytes()),
                sql: query.sql.clone(),
                user_parameter_count: query.parameter_count,
                expanded_parameter_count: rewrite.expanded_parameter_count,
                fetch_mode: query.fetch.as_str().to_owned(),
                row_type: query.row_type.clone(),
                columns: describe
                    .columns()
                    .iter()
                    .map(|column| column.name().to_owned())
                    .collect(),
            });
        }
        Ok(metadata)
    })
}

fn validate_from_query_cache(
    library: &LibraryIr,
    migrations: &str,
    schema_hash: &str,
    queries: &[QuerySpec],
    row_columns: &HashMap<String, HashSet<String>>,
) -> Result<(), String> {
    let cache_path = query_cache_path(library);
    let cache_source = fs::read_to_string(&cache_path).map_err(|error| {
        format!(
            "Dust DB offline query metadata cache is missing or unreadable at `{}`: {error}",
            cache_path.display()
        )
    })?;
    let cache: QueryCache = serde_json::from_str(&cache_source).map_err(|error| {
        format!(
            "Dust DB offline query metadata cache `{}` is invalid: {error}",
            cache_path.display()
        )
    })?;
    if cache.version != QUERY_CACHE_VERSION {
        return Err(format!(
            "Dust DB offline query metadata cache `{}` uses unsupported version {}; run `dust build --db` online first",
            cache_path.display(),
            cache.version
        ));
    }

    for query in queries {
        let rewrite = validate_placeholders(&query.sql, query.parameter_count)?;
        let sql_hash = stable_hash_hex(query.sql.as_bytes());
        let Some(entry) = cache.entries.iter().find(|entry| {
            entry.migrations == migrations
                && entry.schema_hash == schema_hash
                && entry.sql_hash == sql_hash
                && entry.sql == query.sql
                && entry.fetch_mode == query.fetch.as_str()
                && entry.row_type == query.row_type
        }) else {
            return Err(format!(
                "Dust DB offline query metadata cache is missing entry for `{}`; run `dust build --db` online first",
                query.display_name()
            ));
        };
        if entry.user_parameter_count != query.parameter_count {
            return Err(format!(
                "cached SQL metadata for `{}` expects {} parameters but query binds {} args",
                query.display_name(),
                entry.user_parameter_count,
                query.parameter_count
            ));
        }
        if entry.expanded_parameter_count != rewrite.expanded_parameter_count {
            return Err(format!(
                "cached SQL metadata for `{}` has stale placeholder expansion; run `dust build --db` online first",
                query.display_name()
            ));
        }
        validate_cached_columns(query, row_columns, &entry.columns)?;
    }
    Ok(())
}

fn validate_cached_columns(
    query: &QuerySpec,
    row_columns: &HashMap<String, HashSet<String>>,
    returned_columns: &[String],
) -> Result<(), String> {
    let Some(row_type) = query_row_type(query) else {
        return Ok(());
    };
    let Some(required_columns) = row_columns.get(row_type) else {
        return Ok(());
    };
    let returned_columns = returned_columns.iter().collect::<HashSet<_>>();
    if let Some(missing) = required_columns
        .iter()
        .find(|column| !returned_columns.contains(*column))
    {
        return Err(format!(
            "cached SQL metadata for `{}` does not return required column `{missing}` for row `{row_type}`",
            query.display_name()
        ));
    }
    Ok(())
}

fn write_query_cache(library: &LibraryIr, entries: Vec<QueryCacheEntry>) -> Result<(), String> {
    if entries.is_empty() {
        return Ok(());
    }

    let path = query_cache_path(library);
    let mut cache = fs::read_to_string(&path)
        .ok()
        .and_then(|source| serde_json::from_str::<QueryCache>(&source).ok())
        .filter(|cache| cache.version == QUERY_CACHE_VERSION)
        .unwrap_or_default();
    for entry in entries {
        cache.entries.retain(|existing| {
            !(existing.migrations == entry.migrations
                && existing.schema_hash == entry.schema_hash
                && existing.sql_hash == entry.sql_hash
                && existing.fetch_mode == entry.fetch_mode
                && existing.row_type == entry.row_type)
        });
        cache.entries.push(entry);
    }
    cache.entries.sort_by(|left, right| {
        left.migrations
            .cmp(&right.migrations)
            .then(left.schema_hash.cmp(&right.schema_hash))
            .then(left.sql_hash.cmp(&right.sql_hash))
            .then(left.fetch_mode.cmp(&right.fetch_mode))
    });

    let source = serde_json::to_string_pretty(&cache)
        .map_err(|error| format!("failed to encode Dust DB query cache: {error}"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create Dust DB query cache directory `{}`: {error}",
                parent.display()
            )
        })?;
    }
    fs::write(&path, format!("{source}\n")).map_err(|error| {
        format!(
            "failed to write Dust DB query cache `{}`: {error}",
            path.display()
        )
    })
}

fn validate_described_columns(
    query: &QuerySpec,
    row_columns: &HashMap<String, HashSet<String>>,
    describe: &sqlx::Describe<sqlx::Sqlite>,
) -> Result<(), String> {
    if matches!(query.function, QueryFunction::Scalar) && describe.columns().len() != 1 {
        return Err(format!(
            "SQLx query `{}` must return exactly one scalar column",
            query.display_name()
        ));
    }
    let Some(row_type) = query_row_type(query) else {
        return Ok(());
    };
    let Some(required_columns) = row_columns.get(row_type) else {
        return Ok(());
    };
    let returned_columns = describe
        .columns()
        .iter()
        .map(|column| column.name().to_owned())
        .collect::<HashSet<_>>();
    if let Some(missing) = required_columns
        .iter()
        .find(|column| !returned_columns.contains(*column))
    {
        return Err(format!(
            "SQLx query `{}` does not return required column `{missing}` for row `{row_type}`",
            query.display_name()
        ));
    }
    Ok(())
}

fn validate_placeholders(
    sql: &str,
    user_parameter_count: usize,
) -> Result<PlaceholderRewrite, String> {
    let mut rewritten = String::new();
    let mut order = Vec::<usize>::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let bytes = sql.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let ch = sql[i..].chars().next().unwrap_or_default();
        if ch == '\'' && !in_double_quote {
            rewritten.push(ch);
            if in_single_quote && sql[i + ch.len_utf8()..].starts_with('\'') {
                i += ch.len_utf8();
                rewritten.push('\'');
            } else {
                in_single_quote = !in_single_quote;
            }
            i += ch.len_utf8();
            continue;
        }
        if ch == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            rewritten.push(ch);
            i += ch.len_utf8();
            continue;
        }
        if ch == '$' && !in_single_quote && !in_double_quote {
            let mut end = i + 1;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end > i + 1 {
                let index = sql[i + 1..end]
                    .parse::<usize>()
                    .map_err(|_| "invalid SQL placeholder".to_owned())?;
                if index == 0 {
                    return Err("SQL placeholders are 1-based".to_owned());
                }
                order.push(index);
                rewritten.push('?');
                i = end;
                continue;
            }
        }
        rewritten.push(ch);
        i += ch.len_utf8();
    }

    let max = order.iter().copied().max().unwrap_or(0);
    for index in 1..=max {
        if !order.contains(&index) {
            return Err(format!("SQL placeholders must not skip `${index}`"));
        }
    }
    if user_parameter_count != max {
        return Err(format!(
            "query binds {user_parameter_count} args but SQL expects {max} parameters"
        ));
    }
    Ok(PlaceholderRewrite {
        sql: rewritten,
        expanded_parameter_count: order.len(),
    })
}

#[derive(Debug)]
struct PlaceholderRewrite {
    sql: String,
    expanded_parameter_count: usize,
}

fn migration_files(path: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = fs::read_dir(path)
        .map_err(|error| format!("failed to read migrations `{}`: {error}", path.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sql"))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn schema_hash(migrations_path: &Path) -> Result<String, String> {
    let mut hash = StableHash::new();
    for migration in migration_files(migrations_path)? {
        let relative_path = migration
            .strip_prefix(migrations_path)
            .unwrap_or(&migration);
        hash.update(relative_path.to_string_lossy().as_bytes());
        hash.update(b"\0");
        let source = fs::read(&migration).map_err(|error| {
            format!(
                "failed to read migration `{}`: {error}",
                migration.display()
            )
        })?;
        hash.update(&source);
        hash.update(b"\0");
    }
    Ok(hash.finish_hex())
}

fn query_cache_path(library: &LibraryIr) -> PathBuf {
    Path::new(&library.package_root).join(".dart_tool/dust/db_query_cache_v2.json")
}

fn stable_hash_hex(bytes: &[u8]) -> String {
    let mut hash = StableHash::new();
    hash.update(bytes);
    hash.finish_hex()
}

struct StableHash(u64);

impl StableHash {
    const fn new() -> Self {
        Self(1469598103934665603)
    }

    fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(1099511628211);
        }
    }

    fn finish_hex(self) -> String {
        format!("{:016x}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct QueryCache {
    version: u32,
    entries: Vec<QueryCacheEntry>,
}

const QUERY_CACHE_VERSION: u32 = 2;

impl Default for QueryCache {
    fn default() -> Self {
        Self {
            version: QUERY_CACHE_VERSION,
            entries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct QueryCacheEntry {
    migrations: String,
    schema_hash: String,
    sql_hash: String,
    sql: String,
    user_parameter_count: usize,
    expanded_parameter_count: usize,
    fetch_mode: String,
    row_type: Option<String>,
    columns: Vec<String>,
}

fn row_column_map(rows: &[RowClass<'_>]) -> HashMap<String, HashSet<String>> {
    let row_by_name = rows
        .iter()
        .map(|row| (row.class.name.as_str(), row))
        .collect::<HashMap<_, _>>();
    rows.iter()
        .map(|row| {
            (
                row.class.name.clone(),
                collect_row_columns(row, &row_by_name).into_iter().collect(),
            )
        })
        .collect()
}

fn collect_row_columns(
    row: &RowClass<'_>,
    row_by_name: &HashMap<&str, &RowClass<'_>>,
) -> Vec<String> {
    let mut columns = Vec::new();
    for field in &row.class.fields {
        let config = sqlx_config(&field.configs);
        if config.skip {
            continue;
        }
        if config.flatten {
            if let Some(flattened) = field.ty.name().and_then(|name| row_by_name.get(name)) {
                columns.extend(collect_row_columns(flattened, row_by_name));
            }
            continue;
        }
        columns.push(effective_column_name(&row.config, &field.name, &config));
    }
    columns
}

fn is_supported_row_type(ty: &TypeIr) -> bool {
    matches!(
        ty.name(),
        Some("String" | "int" | "double" | "num" | "bool" | "DateTime")
    ) || ty.is_nullable()
        && matches!(
            ty.name(),
            Some("String" | "int" | "double" | "num" | "bool" | "DateTime")
        )
}

fn is_supported_scalar_type(ty: &TypeIr) -> bool {
    matches!(
        ty.name(),
        Some("String" | "int" | "double" | "num" | "bool" | "DateTime")
    )
}

fn query_row_type(query: &QuerySpec) -> Option<&str> {
    matches!(query.function, QueryFunction::As).then(|| query.row_type.as_deref())?
}

fn render_type(ty: &TypeIr) -> String {
    match ty {
        TypeIr::Builtin { kind, nullable } => {
            format!("{}{}", kind.as_str(), if *nullable { "?" } else { "" })
        }
        TypeIr::Named { name, nullable, .. } => {
            format!("{name}{}", if *nullable { "?" } else { "" })
        }
        TypeIr::Function {
            signature,
            nullable,
        } => format!("{signature}{}", if *nullable { "?" } else { "" }),
        TypeIr::Record { shape, nullable } => {
            format!("{shape}{}", if *nullable { "?" } else { "" })
        }
        TypeIr::Dynamic => "dynamic".to_owned(),
        TypeIr::Unknown => "unknown".to_owned(),
    }
}

fn push_duplicate_column(
    class: &ClassIr,
    field: &FieldIr,
    existing: &FieldIr,
    column: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    diagnostics.push(
        Diagnostic::error(format!(
            "duplicate SQL column `{column}` on FromRow class `{}`",
            class.name
        ))
        .with_label(SourceLabel::new(
            field.span.file_id,
            field.span.range,
            format!("field `{}` maps to `{column}`", field.name),
        ))
        .with_label(SourceLabel::new(
            existing.span.file_id,
            existing.span.range,
            format!("field `{}` already maps to `{column}`", existing.name),
        )),
    );
}

fn validate_field_shape(
    class: &ClassIr,
    field: &FieldIr,
    config: &SqlxConfig,
    row_names: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if config.skip
        && !field.has_default
        && config.default_value_source.is_none()
        && !constructor_param_has_default(class, &field.name)
    {
        diagnostics.push(
            Diagnostic::error(format!(
                "field `{}` on `{}` uses `Sqlx(skip: true)` without a default",
                field.name, class.name
            ))
            .with_label(SourceLabel::new(
                field.span.file_id,
                field.span.range,
                "add a Dart field default or `Sqlx(defaultValue: ...)`",
            )),
        );
    }
    if config.flatten {
        let Some(name) = field.ty.name() else {
            diagnostics.push(error_on_field(
                field,
                "flattened SQLx field must use a named FromRow type",
            ));
            return;
        };
        if !row_names.contains(name) {
            diagnostics.push(error_on_field(
                field,
                format!(
                    "flattened SQLx field `{}` must reference an @FromRow class",
                    field.name
                ),
            ));
        }
    }
    if config.json && config.flatten {
        diagnostics.push(error_on_field(
            field,
            "SQLx field cannot use both `json` and `flatten`",
        ));
    }
    if config.try_from_source.is_some() && config.flatten {
        diagnostics.push(error_on_field(
            field,
            "SQLx field cannot use both `tryFrom` and `flatten`",
        ));
    }
    if !config.flatten
        && !config.json
        && config.try_from_source.is_none()
        && !is_supported_row_type(&field.ty)
    {
        diagnostics.push(error_on_field(
            field,
            format!(
                "unsupported SQLx row field type `{}`",
                render_type(&field.ty)
            ),
        ));
    }
}

fn error_on_field(field: &FieldIr, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(message.into()).with_label(SourceLabel::new(
        field.span.file_id,
        field.span.range,
        "unsupported SQLx row mapping",
    ))
}

fn query_error(query: &QuerySpec, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(message.into()).with_label(SourceLabel::new(
        query.span.file_id,
        query.span.range,
        "invalid Dust DB query",
    ))
}

fn constructor_param_has_default(class: &ClassIr, field_name: &str) -> bool {
    class.constructors.iter().any(|constructor| {
        constructor
            .params
            .iter()
            .any(|param| param.name == field_name && param.has_default)
    })
}

#[cfg(test)]
mod tests {
    use super::validate_placeholders;

    #[test]
    fn placeholder_validation_rewrites_repeated_params() {
        let rewrite = validate_placeholders(
            "SELECT '$1' AS label WHERE id = $1 OR owner_id = $1 AND name = $2",
            2,
        )
        .unwrap();

        assert_eq!(
            rewrite.sql,
            "SELECT '$1' AS label WHERE id = ? OR owner_id = ? AND name = ?"
        );
        assert_eq!(rewrite.expanded_parameter_count, 3);
    }

    #[test]
    fn placeholder_validation_rejects_gaps() {
        assert_eq!(
            validate_placeholders("SELECT * FROM users WHERE id = $2", 2).unwrap_err(),
            "SQL placeholders must not skip `$1`"
        );
    }
}
