use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_ir::{ClassIr, FieldIr, LibraryIr, TypeIr};
use either::Either;
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
    if options.offline {
        diagnostics.push(Diagnostic::error(
            "Dust DB offline query metadata cache is missing or not generated yet",
        ));
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

    match run_sqlx_validation(&migrations_path, queries, row_columns) {
        Ok(()) => {}
        Err(error) => diagnostics.push(Diagnostic::error(error)),
    }
}

fn run_sqlx_validation(
    migrations_path: &Path,
    queries: &[QuerySpec<'_>],
    row_columns: &HashMap<String, HashSet<String>>,
) -> Result<(), String> {
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
        }
        Ok(())
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
