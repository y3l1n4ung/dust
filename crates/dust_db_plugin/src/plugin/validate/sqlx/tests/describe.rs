//! Describing a query against a live database and checking its columns.

use super::*;

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
            row_columns: &HashMap::from([("Item".to_owned(), HashSet::from(["id".to_owned()]))]),
            typed_columns: &HashMap::new(),
        },
        &mut warnings,
    )
    .await
    .expect("a valid query describes against PostgreSQL");

    assert_eq!(entries.len(), 1);

    // The whole point of the transaction: the schema the migrations built
    // is gone, and so is the table.
    let leaked: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_schema.schemata WHERE schema_name = 'dust_validation'",
    )
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
    let leaked: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_schema.schemata WHERE schema_name = 'dust_validation'",
    )
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
