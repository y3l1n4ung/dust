use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use dust_diagnostics::Diagnostic;
use dust_plugin_api::{MetadataOutput, ValidationAccess};
use either::Either;
use sqlx::{Column, Connection, Executor, postgres::PgConnection, sqlite::SqliteConnection};

use crate::plugin::{
    DbPluginOptions,
    analysis::PackageDatabase,
    dialect::Dialect,
    migrations::applied_migration_files,
    model::{DbDriver, QueryFunction, QuerySpec},
};

use super::{
    cache::{
        QueryCacheEntry, schema_hash, stable_hash_hex, validate_from_query_cache, write_query_cache,
    },
    query::{query_row_type, validate_placeholders},
};

/// Validates SQL queries through SQLx describe or the offline query cache.
pub(super) fn validate_sqlx_describe(
    library: &dust_ir::DartFileIr,
    db: &PackageDatabase,
    queries: &[QuerySpec],
    row_columns: &HashMap<String, HashSet<String>>,
    options: DbPluginOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // A dialect the engine cannot describe is reported once by the caller; it
    // must not also be described here against the wrong backend.
    if queries.is_empty() || !db.driver.dialect().validates {
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

    if matches!(options.execution.validation, ValidationAccess::Offline) {
        match validate_from_query_cache(
            library,
            db.driver.as_str(),
            &db.migrations,
            &schema_hash,
            queries,
            row_columns,
        ) {
            Ok(()) => {}
            Err(error) => diagnostics.push(Diagnostic::error(error)),
        }
        return;
    }

    match run_sqlx_validation(
        &migrations_path,
        db.driver,
        &db.migrations,
        &schema_hash,
        queries,
        row_columns,
    ) {
        Ok(metadata) => {
            if matches!(options.execution.metadata, MetadataOutput::Write) {
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
    driver: DbDriver,
    migrations: &str,
    schema_hash: &str,
    queries: &[QuerySpec],
    row_columns: &HashMap<String, HashSet<String>>,
) -> Result<Vec<QueryCacheEntry>, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("failed to create SQL validation runtime: {error}"))?;
    // The one place a dialect's backend is chosen; everything past it is
    // generic over the connection.
    runtime.block_on(async move {
        match driver {
            DbDriver::Sqlite3 => {
                let mut conn =
                    connect_for_validation::<SqliteConnection>(driver, Some("sqlite::memory:"))
                        .await?;
                apply_migrations(&mut conn, migrations_path).await?;
                describe_queries(
                    &mut conn,
                    driver.dialect(),
                    migrations,
                    schema_hash,
                    queries,
                    row_columns,
                )
                .await
            }
            DbDriver::Postgres => {
                // No in-memory Postgres, so there is no default to fall back
                // to: validation needs a server or it does not run.
                let mut conn = connect_for_validation::<PgConnection>(driver, None).await?;
                // Migrations are applied inside a transaction that is never
                // committed, so validating leaves the developer's database as
                // it found it. `describe` sees the schema either way.
                let mut tx = conn.begin().await.map_err(|error| {
                    format!("failed to open the SQL validation transaction: {error}")
                })?;
                // Migrations go into a scratch schema, not the developer's own:
                // a database the application has already run against holds the
                // tables, and the first `CREATE TABLE` would fail. The rollback
                // removes the schema with everything in it.
                for setup in [
                    "CREATE SCHEMA dust_validation",
                    "SET LOCAL search_path TO dust_validation",
                ] {
                    (&mut *tx).execute(setup).await.map_err(|error| {
                        format!("failed to prepare the SQL validation schema: {error}")
                    })?;
                }
                apply_migrations(&mut *tx, migrations_path).await?;
                let described = describe_queries(
                    &mut *tx,
                    driver.dialect(),
                    migrations,
                    schema_hash,
                    queries,
                    row_columns,
                )
                .await;
                // A failure to roll back matters more than the describe result.
                tx.rollback().await.map_err(|error| {
                    format!("failed to roll back the SQL validation transaction: {error}")
                })?;
                described
            }
        }
    })
}

/// Opens the database SQL is validated against.
///
/// `DUST_DATABASE_URL` names it. SQLite has an in-memory default because it can
/// build the schema from the migrations alone; PostgreSQL has no equivalent, so
/// it says what is missing rather than failing to connect to nothing.
async fn connect_for_validation<C: Connection>(
    driver: DbDriver,
    fallback: Option<&str>,
) -> Result<C, String> {
    let database_url = match (std::env::var("DUST_DATABASE_URL").ok(), fallback) {
        (Some(url), _) => url,
        (None, Some(fallback)) => fallback.to_owned(),
        (None, None) => {
            return Err(format!(
                "validating SQL for `{}` needs a database: set DUST_DATABASE_URL, or build with \
                 --offline to validate from the committed query cache",
                driver.as_str()
            ));
        }
    };
    C::connect(&database_url).await.map_err(|error| {
        format!("failed to connect SQL validation database `{database_url}`: {error}")
    })
}

/// Applies migration files to the validation database.
async fn apply_migrations<C>(conn: &mut C, migrations_path: &Path) -> Result<(), String>
where
    C: Connection,
    for<'e> &'e mut C: Executor<'e, Database = C::Database>,
{
    for migration in applied_migration_files(migrations_path)? {
        let sql = fs::read_to_string(&migration.path).map_err(|error| {
            format!(
                "failed to read migration `{}`: {error}",
                migration.path.display()
            )
        })?;
        conn.execute(sql.as_str()).await.map_err(|error| {
            format!(
                "failed to apply migration `{}`: {error}",
                migration.path.display()
            )
        })?;
    }
    Ok(())
}

/// Describes all queries and returns metadata suitable for cache writes.
async fn describe_queries<C>(
    conn: &mut C,
    dialect: &Dialect,
    migrations: &str,
    schema_hash: &str,
    queries: &[QuerySpec],
    row_columns: &HashMap<String, HashSet<String>>,
) -> Result<Vec<QueryCacheEntry>, String>
where
    C: Connection,
    for<'e> &'e mut C: Executor<'e, Database = C::Database>,
{
    let mut metadata = Vec::new();
    for query in queries {
        // Unchecked SQL is not described and never enters the cache: its text
        // may be dynamic, and pretending otherwise would put an entry in the
        // committed cache that no build can reproduce.
        if !query.sql_source_static || matches!(query.function, QueryFunction::Unsafe) {
            continue;
        }
        // Arity and gap checking is dialect-independent, so it always runs. Which
        // text the database is asked about is not: SQLite receives `?` and
        // PostgreSQL receives the `$n` as written.
        let rewrite = validate_placeholders(&query.sql, query.parameter_count)?;
        let (described_sql, expected_parameters) = if dialect.rewrites_placeholders {
            (rewrite.sql.as_str(), rewrite.expanded_parameter_count())
        } else {
            (query.sql.as_str(), query.parameter_count)
        };
        let describe = conn
            .describe(described_sql)
            .await
            .map_err(|error| format!("SQLx rejected `{}`: {error}", query.display_name()))?;
        let parameter_count = describe.parameters().map_or(0, |params| match params {
            Either::Left(values) => values.len(),
            Either::Right(count) => count,
        });
        if parameter_count != expected_parameters {
            // The bind list is built from the placeholders Dust replaced, so a
            // disagreement here means the generated call would bind the wrong
            // number of arguments. Show the SQL the database actually parsed:
            // the `$n` that is not where it looks is visible in it.
            return Err(format!(
                "SQLx query `{}` would bind {} arguments to a statement the database reads as \
                 having {parameter_count} parameters. A `$n` was rewritten somewhere it cannot \
                 be bound:\n{}",
                query.display_name(),
                expected_parameters,
                described_sql.trim()
            ));
        }
        validate_described_columns(query, row_columns, &describe)?;
        metadata.push(QueryCacheEntry {
            driver: dialect.name.to_owned(),
            migrations: migrations.to_owned(),
            schema_hash: schema_hash.to_owned(),
            sql_hash: stable_hash_hex(query.sql.as_bytes()),
            sql: query.sql.clone(),
            user_parameter_count: query.parameter_count,
            expanded_parameter_count: expected_parameters,
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
fn validate_described_columns<DB: sqlx::Database>(
    query: &QuerySpec,
    row_columns: &HashMap<String, HashSet<String>>,
    describe: &sqlx::Describe<DB>,
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
