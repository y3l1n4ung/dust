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
mod tests {
    use dust_ir::SpanIr;
    use dust_text::{FileId, TextRange};
    use sqlx::sqlite::SqliteConnection;

    use super::*;
    use crate::plugin::model::{FetchMode, QueryFunction};

    /// SQLite is a real sqlx backend that needs no server, so the paths this
    /// file shares between the two dialects can be exercised in-process. Only
    /// connecting to PostgreSQL genuinely needs one.
    async fn memory_connection() -> SqliteConnection {
        SqliteConnection::connect("sqlite::memory:")
            .await
            .expect("in-memory SQLite must open")
    }

    fn migrations_dir(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("temp dir");
        for (name, sql) in files {
            fs::write(dir.path().join(name), sql).expect("write migration");
        }
        dir
    }

    #[test]
    fn a_url_for_another_driver_is_ignored_rather_than_opened() {
        // One workspace, two drivers, one environment variable. A SQLite
        // project handed a PostgreSQL URL falls back to its own default.
        let chosen = validation_database_url(
            DbDriver::Sqlite3,
            Some("postgres://user@localhost/app?sslmode=disable".to_owned()),
            Some("sqlite::memory:"),
        );

        assert_eq!(chosen.as_deref(), Ok("sqlite::memory:"));
    }

    #[test]
    fn a_url_for_this_driver_wins_over_the_fallback() {
        let chosen = validation_database_url(
            DbDriver::Postgres,
            Some("postgres://user@localhost/app".to_owned()),
            Some("ignored"),
        );

        assert_eq!(chosen.as_deref(), Ok("postgres://user@localhost/app"));
    }

    #[test]
    fn a_driver_with_no_url_and_no_fallback_says_which_scheme_it_needs() {
        let error = validation_database_url(DbDriver::Postgres, None, None)
            .expect_err("PostgreSQL has no in-memory default");

        assert!(error.contains("`postgres`/`postgresql`"), "{error}");
        assert!(error.contains("--offline"), "{error}");
    }

    #[tokio::test]
    async fn migrations_are_applied_in_name_order() {
        // `0002` depends on `0001` having run, so applying them out of order
        // fails rather than passing quietly.
        let dir = migrations_dir(&[
            ("0001_create.sql", "CREATE TABLE items (id INTEGER);"),
            ("0002_alter.sql", "ALTER TABLE items ADD COLUMN name TEXT;"),
        ]);
        let mut conn = memory_connection().await;

        apply_migrations(&mut conn, dir.path())
            .await
            .expect("migrations must apply");

        conn.execute("SELECT id, name FROM items")
            .await
            .expect("both migrations must have run");
    }

    #[tokio::test]
    async fn a_migration_the_database_refuses_names_the_file() {
        let dir = migrations_dir(&[("0001_broken.sql", "CREATE TABLE (;")]);
        let mut conn = memory_connection().await;

        let error = apply_migrations(&mut conn, dir.path())
            .await
            .expect_err("invalid SQL must fail");

        assert!(error.contains("failed to apply migration"), "{error}");
        assert!(error.contains("0001_broken.sql"), "{error}");
    }

    /// One query spec, with the fields a describe test cares about.
    fn query(sql: &str, function: QueryFunction, parameters: usize) -> QuerySpec {
        QuerySpec {
            function,
            fetch: FetchMode::All,
            sql: sql.to_owned(),
            sql_source_static: true,
            row_type: Some("Item".to_owned()),
            scalar_type: None,
            parameter_count: parameters,
            params_source_is_list: false,
            has_row_mapper_argument: false,
            unsafe_sql_allowed: false,
            span: SpanIr::new(FileId::new(7), TextRange::new(0_u32, 1_u32)),
            display_name: Some("Items.all".to_owned()),
        }
    }

    /// Describes [queries] against a schema built from [setup].
    async fn describe(
        setup: &str,
        queries: &[QuerySpec],
        row_columns: HashMap<String, HashSet<String>>,
        typed_columns: HashMap<String, Vec<RowColumn>>,
    ) -> (Result<Vec<QueryCacheEntry>, String>, Vec<String>) {
        let mut conn = memory_connection().await;
        conn.execute(setup).await.expect("schema must build");
        let mut warnings = Vec::new();
        let result = describe_queries(
            &mut conn,
            &DescribeRequest {
                dialect: DbDriver::Sqlite3.dialect(),
                migrations: "./migrations",
                schema_hash: "hash",
                queries,
                row_columns: &row_columns,
                typed_columns: &typed_columns,
            },
            &mut warnings,
        )
        .await;
        (result, warnings)
    }

    const ITEMS: &str = "CREATE TABLE items (id INTEGER NOT NULL, name TEXT);";

    #[tokio::test]
    async fn a_described_query_becomes_a_cache_entry() {
        let (result, warnings) = describe(
            ITEMS,
            &[query(
                "SELECT id FROM items WHERE id = $1",
                QueryFunction::As,
                1,
            )],
            HashMap::from([("Item".to_owned(), HashSet::from(["id".to_owned()]))]),
            HashMap::new(),
        )
        .await;

        let entries = result.expect("a valid query describes");
        assert_eq!(entries.len(), 1);
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[tokio::test]
    async fn a_query_missing_a_column_the_row_needs_is_reported() {
        let (result, _) = describe(
            ITEMS,
            &[query("SELECT id FROM items", QueryFunction::As, 0)],
            HashMap::from([(
                "Item".to_owned(),
                HashSet::from(["id".to_owned(), "name".to_owned()]),
            )]),
            HashMap::new(),
        )
        .await;

        let error = result.expect_err("a missing column must be reported");
        assert!(
            error.contains("does not return required column `name`"),
            "{error}"
        );
    }

    #[tokio::test]
    async fn a_scalar_query_returning_two_columns_is_reported() {
        let (result, _) = describe(
            ITEMS,
            &[query(
                "SELECT id, name FROM items",
                QueryFunction::Scalar,
                0,
            )],
            HashMap::new(),
            HashMap::new(),
        )
        .await;

        let error = result.expect_err("a scalar reads one column");
        assert!(error.contains("exactly one scalar column"), "{error}");
    }

    #[tokio::test]
    async fn a_column_read_into_the_wrong_dart_type_warns() {
        // A warning rather than an error: the accepted-type table is
        // deliberately permissive, so a finding here is worth reading rather
        // than worth failing a build over.
        let (result, warnings) = describe(
            ITEMS,
            &[query("SELECT name FROM items", QueryFunction::As, 0)],
            HashMap::new(),
            HashMap::from([(
                "Item".to_owned(),
                vec![RowColumn {
                    name: "name".to_owned(),
                    dart_type: "int".to_owned(),
                    nullable: false,
                }],
            )]),
        )
        .await;

        result.expect("a type disagreement is a warning, not an error");
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("name"), "{}", warnings[0]);
    }

    #[tokio::test]
    async fn sql_the_database_rejects_names_the_query() {
        let (result, _) = describe(
            ITEMS,
            &[query("SELECT nope FROM items", QueryFunction::As, 0)],
            HashMap::new(),
            HashMap::new(),
        )
        .await;

        let error = result.expect_err("unknown column must fail");
        assert!(error.contains("Items.all"), "{error}");
    }

    #[tokio::test]
    async fn unchecked_and_dynamic_sql_are_never_described() {
        // Neither may enter the committed cache: one has text a build cannot
        // reproduce, and the other is deliberately outside validation.
        let mut dynamic = query("SELECT id FROM items", QueryFunction::As, 0);
        dynamic.sql_source_static = false;
        let unchecked = query("SELECT whatever", QueryFunction::Unsafe, 0);

        let (result, warnings) =
            describe(ITEMS, &[dynamic, unchecked], HashMap::new(), HashMap::new()).await;

        assert!(result.expect("skipping is not a failure").is_empty());
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    /// The database a PostgreSQL test may write to, when one is named.
    ///
    /// There is no in-memory PostgreSQL, so these cannot bring their own. They
    /// return rather than fail when `DUST_DATABASE_URL` is unset, and the CI
    /// job that has a server is the one that runs them.
    fn postgres_url() -> Option<String> {
        std::env::var("DUST_DATABASE_URL")
            .ok()
            .filter(|url| DbDriver::Postgres.dialect().accepts_url(url))
    }

    #[tokio::test]
    async fn postgres_describes_inside_a_scratch_schema_and_leaves_no_trace() {
        let Some(url) = postgres_url() else {
            return;
        };
        let mut conn = PgConnection::connect(&url)
            .await
            .expect("DUST_DATABASE_URL must connect");
        let dir = migrations_dir(&[(
            "0001_create.sql",
            "CREATE TABLE scratch_items (id BIGINT NOT NULL, name TEXT);",
        )]);
        let mut warnings = Vec::new();

        let entries = describe_in_scratch_schema(
            &mut conn,
            dir.path(),
            &DescribeRequest {
                dialect: DbDriver::Postgres.dialect(),
                migrations: "./migrations",
                schema_hash: "hash",
                queries: &[query(
                    "SELECT id FROM scratch_items WHERE id = $1",
                    QueryFunction::As,
                    1,
                )],
                row_columns: &HashMap::from([(
                    "Item".to_owned(),
                    HashSet::from(["id".to_owned()]),
                )]),
                typed_columns: &HashMap::new(),
            },
            &mut warnings,
        )
        .await
        .expect("a valid query describes against PostgreSQL");

        assert_eq!(entries.len(), 1);

        // The whole point of the transaction: the schema the migrations built
        // is gone, and so is the table.
        let leaked: i64 =
            sqlx::query_scalar("SELECT count(*) FROM information_schema.schemata WHERE schema_name = 'dust_validation'")
                .fetch_one(&mut conn)
                .await
                .expect("catalog query");
        assert_eq!(leaked, 0, "validation left its scratch schema behind");
    }

    #[tokio::test]
    async fn postgres_reports_sql_the_server_rejects() {
        let Some(url) = postgres_url() else {
            return;
        };
        let mut conn = PgConnection::connect(&url)
            .await
            .expect("DUST_DATABASE_URL must connect");
        let dir = migrations_dir(&[("0001_create.sql", "CREATE TABLE scratch_items (id BIGINT);")]);
        let mut warnings = Vec::new();

        let error = describe_in_scratch_schema(
            &mut conn,
            dir.path(),
            &DescribeRequest {
                dialect: DbDriver::Postgres.dialect(),
                migrations: "./migrations",
                schema_hash: "hash",
                queries: &[query(
                    "SELECT nope FROM scratch_items",
                    QueryFunction::As,
                    0,
                )],
                row_columns: &HashMap::new(),
                typed_columns: &HashMap::new(),
            },
            &mut warnings,
        )
        .await
        .expect_err("an unknown column must be reported");

        assert!(error.contains("Items.all"), "{error}");

        // Rolled back even though describing failed.
        let leaked: i64 =
            sqlx::query_scalar("SELECT count(*) FROM information_schema.schemata WHERE schema_name = 'dust_validation'")
                .fetch_one(&mut conn)
                .await
                .expect("catalog query");
        assert_eq!(
            leaked, 0,
            "a failed describe left its scratch schema behind"
        );
    }

    #[tokio::test]
    async fn postgres_needs_a_url_and_says_so() {
        // The one dialect with no in-memory default. Reached through the real
        // entry point, so the message a developer sees is the one asserted.
        let error = connect_for_validation::<PgConnection>(DbDriver::Postgres, None)
            .await
            .err();

        // With a URL set this connects; without one it explains itself. Both
        // are correct, and only the second is worth asserting on.
        if postgres_url().is_none() {
            let error = error.expect("no URL and no fallback cannot connect");
            assert!(error.contains("`postgres`/`postgresql`"), "{error}");
        }
    }

    #[tokio::test]
    async fn a_migrations_directory_with_nothing_in_it_is_not_an_error() {
        let dir = migrations_dir(&[]);
        let mut conn = memory_connection().await;

        apply_migrations(&mut conn, dir.path())
            .await
            .expect("no migrations is not a failure");
    }
}
