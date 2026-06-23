use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use dust_diagnostics::Diagnostic;
use either::Either;
use sqlx::{Column, Connection, Executor, sqlite::SqliteConnection};

use crate::plugin::{
    DbPluginOptions,
    model::{DatabaseClass, DbDriver, QueryFunction, QuerySpec},
};

use super::{
    cache::{
        QueryCacheEntry, migration_files, schema_hash, stable_hash_hex, validate_from_query_cache,
        write_query_cache,
    },
    query::{query_row_type, validate_placeholders},
};

/// Validates SQL queries through SQLx describe or the offline query cache.
pub(super) fn validate_sqlx_describe(
    library: &dust_ir::DartFileIr,
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

/// Runs online SQLx validation inside a current-thread Tokio runtime.
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
        let mut conn = connect_sqlite_for_validation().await?;
        apply_migrations(&mut conn, migrations_path).await?;
        describe_queries(&mut conn, migrations, schema_hash, queries, row_columns).await
    })
}

/// Opens the SQLite database used for SQL validation.
async fn connect_sqlite_for_validation() -> Result<SqliteConnection, String> {
    let database_url =
        std::env::var("DUST_DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_owned());
    SqliteConnection::connect(&database_url)
        .await
        .map_err(|error| {
            format!("failed to connect SQL validation database `{database_url}`: {error}")
        })
}

/// Applies migration files to the validation database.
async fn apply_migrations(
    conn: &mut SqliteConnection,
    migrations_path: &Path,
) -> Result<(), String> {
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
    Ok(())
}

/// Describes all queries and returns metadata suitable for cache writes.
async fn describe_queries(
    conn: &mut SqliteConnection,
    migrations: &str,
    schema_hash: &str,
    queries: &[QuerySpec],
    row_columns: &HashMap<String, HashSet<String>>,
) -> Result<Vec<QueryCacheEntry>, String> {
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
        if parameter_count != rewrite.expanded_parameter_count() {
            return Err(format!(
                "SQLx query `{}` expects {parameter_count} expanded parameters but Dust rewrote {} placeholders",
                query.display_name(),
                rewrite.expanded_parameter_count()
            ));
        }
        validate_described_columns(query, row_columns, &describe)?;
        metadata.push(QueryCacheEntry {
            migrations: migrations.to_owned(),
            schema_hash: schema_hash.to_owned(),
            sql_hash: stable_hash_hex(query.sql.as_bytes()),
            sql: query.sql.clone(),
            user_parameter_count: query.parameter_count,
            expanded_parameter_count: rewrite.expanded_parameter_count(),
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
}

/// Validates SQLx-described columns against scalar and row requirements.
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
