use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_ir::{ClassIr, FieldIr, LibraryIr, TypeIr};
use either::Either;
use serde::{Deserialize, Serialize};
use sqlx::{Column, Connection, Executor, sqlite::SqliteConnection};

use super::{
    DbPluginOptions,
    model::{FetchKind, QuerySpec, RowClass, SqlxConfig},
    parse::{dust_db_classes, effective_column_name, row_classes, sqlx_config},
};

pub(crate) fn validate_db_library(
    library: &LibraryIr,
    options: DbPluginOptions,
) -> Vec<Diagnostic> {
    let rows = row_classes(library);
    let mut diagnostics = Vec::new();
    validate_rows(&rows, &mut diagnostics);
    validate_repositories(library, options, &rows, &mut diagnostics);
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

fn validate_repositories(
    library: &LibraryIr,
    options: DbPluginOptions,
    rows: &[RowClass<'_>],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let row_columns = row_column_map(rows);
    for db in dust_db_classes(library) {
        if db.migrations.trim().is_empty() {
            diagnostics.push(Diagnostic::error(format!(
                "DustDb class `{}` must provide a migrations path",
                db.class.name
            )));
        }
        for query in &db.queries {
            let placeholder_count = query.sql.matches('?').count();
            if placeholder_count != query.args.len() {
                diagnostics.push(
                    Diagnostic::error(format!(
                        "query method `{}` binds {} args but SQL contains {} placeholders",
                        query.method.name,
                        query.args.len(),
                        placeholder_count
                    ))
                    .with_label(SourceLabel::new(
                        query.method.span.file_id,
                        query.method.span.range,
                        "make `$fetch` args match `?` placeholders",
                    )),
                );
            }
            validate_query_return(query, &row_columns, diagnostics);
        }
        validate_sqlx_describe(
            library,
            &db.migrations,
            &db.queries,
            &row_columns,
            options,
            diagnostics,
        );
    }
}

fn validate_query_return(
    query: &QuerySpec<'_>,
    row_columns: &HashMap<String, HashSet<String>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match query.fetch {
        FetchKind::One | FetchKind::InsertOne => {
            if query_item_type(query)
                .and_then(|name| row_columns.get(name))
                .is_none()
            {
                diagnostics.push(query_error(
                    query,
                    "row query must return a Future<T?> or Future<T> where T is @FromRow",
                ));
            }
        }
        FetchKind::All => {
            if list_query_item_type(query)
                .and_then(|name| row_columns.get(name))
                .is_none()
            {
                diagnostics.push(query_error(
                    query,
                    "all query must return Future<List<T>> where T is @FromRow",
                ));
            }
        }
        FetchKind::Stream => {
            if stream_query_item_type(query)
                .and_then(|name| row_columns.get(name))
                .is_none()
            {
                diagnostics.push(query_error(
                    query,
                    "stream query must return Stream<T> where T is @FromRow",
                ));
            }
        }
        FetchKind::Scalar | FetchKind::Execute => {}
    }
}

fn validate_sqlx_describe(
    library: &LibraryIr,
    migrations: &str,
    queries: &[QuerySpec<'_>],
    row_columns: &HashMap<String, HashSet<String>>,
    options: DbPluginOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if queries.is_empty() {
        return;
    }
    let migrations_path = Path::new(&library.package_root).join(migrations);
    if !migrations_path.exists() {
        diagnostics.push(Diagnostic::error(format!(
            "DustDb migrations path `{}` does not exist",
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
        match validate_from_query_cache(library, migrations, &schema_hash, queries, row_columns) {
            Ok(()) => {}
            Err(error) => diagnostics.push(Diagnostic::error(error)),
        }
        return;
    }

    match run_sqlx_validation(
        &migrations_path,
        migrations,
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
    queries: &[QuerySpec<'_>],
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
            let describe = conn
                .describe(query.sql.as_str())
                .await
                .map_err(|error| format!("SQLx rejected `{}`: {error}", query.method.name))?;
            let parameter_count = describe.parameters().map_or(0, |params| match params {
                Either::Left(values) => values.len(),
                Either::Right(count) => count,
            });
            if parameter_count != query.args.len() {
                return Err(format!(
                    "SQLx query `{}` expects {parameter_count} parameters but method binds {} args",
                    query.method.name,
                    query.args.len()
                ));
            }
            validate_described_columns(query, row_columns, &describe)?;
            metadata.push(QueryCacheEntry {
                migrations: migrations.to_owned(),
                schema_hash: schema_hash.to_owned(),
                sql_hash: stable_hash_hex(query.sql.as_bytes()),
                sql: query.sql.clone(),
                parameter_count,
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
    queries: &[QuerySpec<'_>],
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
        let sql_hash = stable_hash_hex(query.sql.as_bytes());
        let Some(entry) = cache.entries.iter().find(|entry| {
            entry.migrations == migrations
                && entry.schema_hash == schema_hash
                && entry.sql_hash == sql_hash
                && entry.sql == query.sql
        }) else {
            return Err(format!(
                "Dust DB offline query metadata cache is missing entry for `{}`; run `dust build --db` online first",
                query.method.name
            ));
        };
        if entry.parameter_count != query.args.len() {
            return Err(format!(
                "cached SQL metadata for `{}` expects {} parameters but method binds {} args",
                query.method.name,
                entry.parameter_count,
                query.args.len()
            ));
        }
        validate_cached_columns(query, row_columns, &entry.columns)?;
    }
    Ok(())
}

fn validate_cached_columns(
    query: &QuerySpec<'_>,
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
            query.method.name
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
        .unwrap_or_default();
    for entry in entries {
        cache.entries.retain(|existing| {
            !(existing.migrations == entry.migrations
                && existing.schema_hash == entry.schema_hash
                && existing.sql_hash == entry.sql_hash)
        });
        cache.entries.push(entry);
    }
    cache.entries.sort_by(|left, right| {
        left.migrations
            .cmp(&right.migrations)
            .then(left.schema_hash.cmp(&right.schema_hash))
            .then(left.sql_hash.cmp(&right.sql_hash))
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
    query: &QuerySpec<'_>,
    row_columns: &HashMap<String, HashSet<String>>,
    describe: &sqlx::Describe<sqlx::Sqlite>,
) -> Result<(), String> {
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
            query.method.name
        ));
    }
    Ok(())
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
    Path::new(&library.package_root).join(".dart_tool/dust/db_query_cache_v1.json")
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

const QUERY_CACHE_VERSION: u32 = 1;

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
    parameter_count: usize,
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

fn query_item_type<'a>(query: &'a QuerySpec<'a>) -> Option<&'a str> {
    if query.method.return_type.is_named("Future") && query.method.return_type.args().len() == 1 {
        return query.method.return_type.args()[0].name();
    }
    None
}

fn query_row_type<'a>(query: &'a QuerySpec<'a>) -> Option<&'a str> {
    match query.fetch {
        FetchKind::One | FetchKind::InsertOne => query_item_type(query),
        FetchKind::All => list_query_item_type(query),
        FetchKind::Stream => stream_query_item_type(query),
        FetchKind::Scalar | FetchKind::Execute => None,
    }
}

fn list_query_item_type<'a>(query: &'a QuerySpec<'a>) -> Option<&'a str> {
    if query.method.return_type.is_named("Future") && query.method.return_type.args().len() == 1 {
        let list = &query.method.return_type.args()[0];
        if list.is_named("List") && list.args().len() == 1 {
            return list.args()[0].name();
        }
    }
    None
}

fn stream_query_item_type<'a>(query: &'a QuerySpec<'a>) -> Option<&'a str> {
    if query.method.return_type.is_named("Stream") && query.method.return_type.args().len() == 1 {
        return query.method.return_type.args()[0].name();
    }
    None
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

fn error_on_field(field: &FieldIr, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(message.into()).with_label(SourceLabel::new(
        field.span.file_id,
        field.span.range,
        "unsupported SQLx row mapping",
    ))
}

fn query_error(query: &QuerySpec<'_>, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(message.into()).with_label(SourceLabel::new(
        query.method.span.file_id,
        query.method.span.range,
        "invalid Dust DB query method",
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
