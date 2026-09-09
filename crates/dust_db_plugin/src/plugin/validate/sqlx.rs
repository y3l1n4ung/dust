use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use dust_diagnostics::Diagnostic;
use dust_plugin_api::{MetadataOutput, ValidationAccess};
use sqlx::{Connection, Executor, postgres::PgConnection, sqlite::SqliteConnection};

use crate::plugin::{
    DbPluginOptions,
    analysis::{PackageDatabase, RowColumn},
    dialect::Dialect,
    migrations::applied_migration_files,
    model::{DbDriver, QuerySpec},
};

use super::{
    cache::{QueryCacheEntry, validate_from_query_cache, write_query_cache},
    describe::describe_queries,
    hash::schema_hash,
};

/// Validates SQL queries through SQLx describe or the offline query cache.
pub(super) fn validate_sqlx_describe(
    library: &dust_ir::DartFileIr,
    db: &PackageDatabase,
    queries: &[QuerySpec],
    row_columns: &HashMap<String, HashSet<String>>,
    typed_columns: &HashMap<String, Vec<RowColumn>>,
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
            db.driver.dialect(),
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

    // Type and nullability findings are warnings, so they are collected rather
    // than short-circuiting the describe run.
    let mut warnings = Vec::new();
    let validated = run_sqlx_validation(
        &migrations_path,
        &DescribeRequest {
            dialect: db.driver.dialect(),
            migrations: &db.migrations,
            schema_hash: &schema_hash,
            queries,
            row_columns,
            typed_columns,
        },
        &mut warnings,
    );
    diagnostics.extend(warnings.into_iter().map(Diagnostic::warning));
    match validated {
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
/// Everything a describe run needs about the queries it is checking.
pub(super) struct DescribeRequest<'a> {
    /// The database being described against.
    pub(super) dialect: &'a Dialect,
    /// Migration directory, as the cache records it.
    pub(super) migrations: &'a str,
    /// Hash of the migrations the schema came from.
    pub(super) schema_hash: &'a str,
    /// Queries to describe.
    pub(super) queries: &'a [QuerySpec],
    /// Columns each row class requires, by name.
    pub(super) row_columns: &'a HashMap<String, HashSet<String>>,
    /// Columns each row class requires, with the field behind each.
    pub(super) typed_columns: &'a HashMap<String, Vec<RowColumn>>,
}

/// Runs SQLx validation inside a current-thread Tokio runtime.
fn run_sqlx_validation(
    migrations_path: &Path,
    request: &DescribeRequest<'_>,
    warnings: &mut Vec<String>,
) -> Result<Vec<QueryCacheEntry>, String> {
    let driver = request.dialect.driver;
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
                describe_queries(&mut conn, request, warnings).await
            }
            DbDriver::Postgres => {
                // No in-memory Postgres, so there is no default to fall back
                // to: validation needs a server or it does not run.
                let mut conn = connect_for_validation::<PgConnection>(driver, None).await?;
                describe_in_scratch_schema(&mut conn, migrations_path, request, warnings).await
            }
        }
    })
}

/// Describes queries against a throwaway schema, leaving the database as found.
///
/// Split out of the match arm so it can be tested: everything here needs a
/// server, and a test that has one can call it directly rather than reaching
/// through a whole plugin run.
async fn describe_in_scratch_schema(
    conn: &mut PgConnection,
    migrations_path: &Path,
    request: &DescribeRequest<'_>,
    warnings: &mut Vec<String>,
) -> Result<Vec<QueryCacheEntry>, String> {
    // Migrations are applied inside a transaction that is never committed, so
    // validating leaves the developer's database as it found it. `describe`
    // sees the schema either way.
    let mut tx = conn
        .begin()
        .await
        .map_err(|error| format!("failed to open the SQL validation transaction: {error}"))?;
    // Migrations go into a scratch schema, not the developer's own: a database
    // the application has already run against holds the tables, and the first
    // `CREATE TABLE` would fail. The rollback removes the schema with
    // everything in it.
    for setup in [
        "CREATE SCHEMA dust_validation",
        "SET LOCAL search_path TO dust_validation",
    ] {
        (&mut *tx)
            .execute(setup)
            .await
            .map_err(|error| format!("failed to prepare the SQL validation schema: {error}"))?;
    }
    apply_migrations(&mut *tx, migrations_path).await?;
    let described = describe_queries(&mut *tx, request, warnings).await;
    // A failure to roll back matters more than the describe result.
    tx.rollback()
        .await
        .map_err(|error| format!("failed to roll back the SQL validation transaction: {error}"))?;
    described
}

/// Opens the database SQL is validated against.
///
/// `DUST_DATABASE_URL` names it. SQLite has an in-memory default because it can
/// build the schema from the migrations alone; PostgreSQL has no equivalent, so
/// it says what is missing rather than failing to connect to nothing.
///
/// A URL for another driver is ignored rather than opened. One workspace can
/// hold projects on both drivers while one environment variable names one
/// database, and a SQLite project handed a PostgreSQL URL would otherwise fail
/// with whatever the other driver's URL parser disliked.
async fn connect_for_validation<C: Connection>(
    driver: DbDriver,
    fallback: Option<&str>,
) -> Result<C, String> {
    let database_url =
        validation_database_url(driver, std::env::var("DUST_DATABASE_URL").ok(), fallback)?;
    C::connect(&database_url).await.map_err(|error| {
        format!("failed to connect SQL validation database `{database_url}`: {error}")
    })
}

/// Chooses the database URL to validate against.
///
/// Separate from connecting so it can be tested without a process-wide
/// environment variable: the choice is the part with rules in it, and reading
/// `DUST_DATABASE_URL` inside a test would race every other test in the binary.
fn validation_database_url(
    driver: DbDriver,
    named: Option<String>,
    fallback: Option<&str>,
) -> Result<String, String> {
    let dialect = driver.dialect();
    let named = named.filter(|url| dialect.accepts_url(url));
    match (named, fallback) {
        (Some(url), _) => Ok(url),
        (None, Some(fallback)) => Ok(fallback.to_owned()),
        (None, None) => Err(format!(
            "validating SQL for `{}` needs a database: set DUST_DATABASE_URL to a `{}` URL, \
             or build with --offline to validate from the committed query cache",
            driver.as_str(),
            dialect.url_schemes.join("`/`")
        )),
    }
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

#[cfg(test)]
#[path = "sqlx/tests.rs"]
mod tests;
