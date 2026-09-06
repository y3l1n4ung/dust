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
    analysis::{PackageDatabase, RowColumn},
    column_alias::{NullabilityOverride, parse_column_alias},
    column_types::accepts,
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
struct DescribeRequest<'a> {
    /// The database being described against.
    dialect: &'a Dialect,
    /// Migration directory, as the cache records it.
    migrations: &'a str,
    /// Hash of the migrations the schema came from.
    schema_hash: &'a str,
    /// Queries to describe.
    queries: &'a [QuerySpec],
    /// Columns each row class requires, by name.
    row_columns: &'a HashMap<String, HashSet<String>>,
    /// Columns each row class requires, with the field behind each.
    typed_columns: &'a HashMap<String, Vec<RowColumn>>,
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
                let described = describe_queries(&mut *tx, request, warnings).await;
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
///
/// A URL for another driver is ignored rather than opened. One workspace can
/// hold projects on both drivers while one environment variable names one
/// database, and a SQLite project handed a PostgreSQL URL would otherwise fail
/// with whatever the other driver's URL parser disliked.
async fn connect_for_validation<C: Connection>(
    driver: DbDriver,
    fallback: Option<&str>,
) -> Result<C, String> {
    let dialect = driver.dialect();
    let named = std::env::var("DUST_DATABASE_URL")
        .ok()
        .filter(|url| dialect.accepts_url(url));
    let database_url = match (named, fallback) {
        (Some(url), _) => url,
        (None, Some(fallback)) => fallback.to_owned(),
        (None, None) => {
            return Err(format!(
                "validating SQL for `{}` needs a database: set DUST_DATABASE_URL to a `{}` URL, \
                 or build with --offline to validate from the committed query cache",
                driver.as_str(),
                dialect.url_schemes.join("`/`")
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
    request: &DescribeRequest<'_>,
    warnings: &mut Vec<String>,
) -> Result<Vec<QueryCacheEntry>, String>
where
    C: Connection,
    for<'e> &'e mut C: Executor<'e, Database = C::Database>,
{
    let DescribeRequest {
        dialect,
        migrations,
        schema_hash,
        queries,
        row_columns,
        typed_columns,
    } = *request;
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
        warnings.extend(describe_column_warnings(
            query,
            dialect,
            typed_columns,
            &describe,
        ));
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
                .map(|column| parse_column_alias(column.name()).name)
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
    // A `foo!` or `foo?` alias is a nullability override, and the marker is not
    // part of the name a row class spells.
    let returned_columns = describe
        .columns()
        .iter()
        .map(|column| parse_column_alias(column.name()).name)
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

/// Reports described types and nullability that disagree with the row class.
///
/// Warnings rather than errors: the accepted type table is deliberately
/// permissive, and a dialect whose inference is wrong about nullability has the
/// column-alias overrides as its answer. A finding here is worth reading, not
/// worth failing a build over yet.
fn describe_column_warnings<DB: sqlx::Database>(
    query: &QuerySpec,
    dialect: &Dialect,
    typed_columns: &HashMap<String, Vec<RowColumn>>,
    describe: &sqlx::Describe<DB>,
) -> Vec<String> {
    let mut warnings = Vec::new();
    let Some(row_type) = query_row_type(query) else {
        return warnings;
    };
    let Some(fields) = typed_columns.get(row_type) else {
        return warnings;
    };

    for (index, column) in describe.columns().iter().enumerate() {
        let alias = parse_column_alias(column.name());
        let Some(field) = fields.iter().find(|field| field.name == alias.name) else {
            continue;
        };

        let sql_type = column.type_info().to_string();
        if !accepts(dialect.driver, &field.dart_type, &sql_type) {
            warnings.push(format!(
                "SQLx query `{}` reads column `{}` of type `{sql_type}` into `{}`, which cannot hold it",
                query.display_name(),
                alias.name,
                field.dart_type,
            ));
        }

        // An override says what the database could not know, so it settles the
        // question. Inference is only consulted for a dialect whose inference
        // is worth consulting.
        let described_nullable = match alias.nullability {
            NullabilityOverride::NotNull => Some(false),
            NullabilityOverride::Nullable => Some(true),
            NullabilityOverride::Inferred if dialect.checks_nullability => describe.nullable(index),
            NullabilityOverride::Inferred => None,
        };
        if described_nullable == Some(true) && !field.nullable {
            warnings.push(format!(
                "SQLx query `{}` reads nullable column `{}` into non-nullable `{}`. Make the field nullable, or write `as \"{}!\"` if the database is wrong about it",
                query.display_name(),
                alias.name,
                field.dart_type,
                alias.name,
            ));
        }
    }
    warnings
}
