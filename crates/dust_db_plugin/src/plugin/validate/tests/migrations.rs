//! Migration ordering, reversible pairs, and the offline query cache.

use super::*;

#[test]
fn migration_files_are_sorted_and_schema_hash_is_stable() {
    let root = temp_root("migrations");
    fs::create_dir_all(root.join("migrations")).unwrap();
    fs::write(root.join("migrations/002_second.sql"), "SELECT 2;\n").unwrap();
    fs::write(root.join("migrations/001_first.sql"), "SELECT 1;\n").unwrap();

    let files = applied_migration_files(&root.join("migrations")).unwrap();
    let names = files
        .iter()
        .map(|migration| migration.name.clone())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["001_first.sql", "002_second.sql"]);
    assert_eq!(
        schema_hash(&root.join("migrations")).unwrap(),
        schema_hash(&root.join("migrations")).unwrap()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn reversible_migration_files_apply_only_up_files() {
    let root = temp_root("reversible_migrations");
    fs::create_dir_all(root.join("migrations")).unwrap();
    fs::write(
        root.join("migrations/001_create_users.up.sql"),
        "CREATE TABLE users(id INTEGER PRIMARY KEY);\n",
    )
    .unwrap();
    fs::write(
        root.join("migrations/001_create_users.down.sql"),
        "DROP TABLE users;\n",
    )
    .unwrap();
    fs::write(root.join("migrations/002_seed.sql"), "SELECT 1;\n").unwrap();

    let files = applied_migration_files(&root.join("migrations")).unwrap();
    let names = files
        .iter()
        .map(|migration| migration.name.clone())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["001_create_users.up.sql", "002_seed.sql"]);

    let schema = schema_hash(&root.join("migrations")).unwrap();
    fs::write(
        root.join("migrations/001_create_users.down.sql"),
        "DROP TABLE users; SELECT 1;\n",
    )
    .unwrap();
    assert_eq!(schema_hash(&root.join("migrations")).unwrap(), schema);

    fs::write(
        root.join("migrations/001_create_users.up.sql"),
        "CREATE TABLE users(id INTEGER PRIMARY KEY, name TEXT);\n",
    )
    .unwrap();
    assert_ne!(schema_hash(&root.join("migrations")).unwrap(), schema);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn reversible_migration_files_reject_incomplete_pairs() {
    let root = temp_root("orphan_down");
    fs::create_dir_all(root.join("migrations")).unwrap();
    fs::write(
        root.join("migrations/001_create_users.down.sql"),
        "DROP TABLE users;\n",
    )
    .unwrap();

    assert!(
        applied_migration_files(&root.join("migrations"))
            .unwrap_err()
            .contains(".down.sql file without matching .up.sql file")
    );
    let _ = fs::remove_dir_all(root);

    let root = temp_root("orphan_up");
    fs::create_dir_all(root.join("migrations")).unwrap();
    fs::write(
        root.join("migrations/001_create_users.up.sql"),
        "CREATE TABLE users(id INTEGER PRIMARY KEY);\n",
    )
    .unwrap();

    assert!(
        applied_migration_files(&root.join("migrations"))
            .unwrap_err()
            .contains(".up.sql file without matching .down.sql file")
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn reversible_migration_files_reject_simple_duplicate_ids() {
    let root = temp_root("duplicate_migrations");
    fs::create_dir_all(root.join("migrations")).unwrap();
    fs::write(root.join("migrations/001_users.sql"), "SELECT 1;\n").unwrap();
    fs::write(root.join("migrations/001_users.up.sql"), "SELECT 2;\n").unwrap();
    fs::write(root.join("migrations/001_users.down.sql"), "SELECT 3;\n").unwrap();

    assert!(
        applied_migration_files(&root.join("migrations"))
            .unwrap_err()
            .contains("both simple and reversible files")
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn offline_query_cache_validates_shape_and_staleness() {
    let root = temp_root("cache");
    let library = library(&root);
    let cache_path = query_cache_path(&library);
    fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
    let query = query(QueryFunction::As, FetchMode::One);
    let cache = QueryCache {
        version: QUERY_CACHE_VERSION,
        entries: vec![QueryCacheEntry {
            driver: "sqlite3".to_owned(),
            migrations: "./migrations".to_owned(),
            schema_hash: "schema".to_owned(),
            sql_hash: crate::plugin::validate::hash::stable_hash_hex(query.sql.as_bytes()),
            sql: query.sql.clone(),
            user_parameter_count: 1,
            expanded_parameter_count: 1,
            fetch_mode: "one".to_owned(),
            row_type: Some("UserRow".to_owned()),
            columns: vec!["id".to_owned()],
        }],
    };
    fs::write(&cache_path, serde_json::to_string_pretty(&cache).unwrap()).unwrap();

    let mut row_columns = HashMap::new();
    row_columns.insert("UserRow".to_owned(), HashSet::from(["id".to_owned()]));
    assert_eq!(
        validate_from_query_cache(
            &library,
            DbDriver::Sqlite3.dialect(),
            "./migrations",
            "schema",
            std::slice::from_ref(&query),
            &row_columns,
        ),
        Ok(())
    );
    assert!(
        validate_from_query_cache(
            &library,
            DbDriver::Sqlite3.dialect(),
            "./migrations",
            "stale",
            std::slice::from_ref(&query),
            &row_columns,
        )
        .unwrap_err()
        .contains("missing entry")
    );
    // A cache written against SQLite cannot answer for Postgres, and the error
    // has to say so rather than claim the query was never described.
    let mismatch = validate_from_query_cache(
        &library,
        DbDriver::Postgres.dialect(),
        "./migrations",
        "schema",
        &[query],
        &row_columns,
    )
    .unwrap_err();
    assert!(
        mismatch.contains("written for driver `sqlite3`"),
        "{mismatch}"
    );
    assert!(mismatch.contains("targets `postgres`"), "{mismatch}");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn query_cache_path_is_committed_next_to_the_package() {
    let root = temp_root("cache_path");
    let library = library(&root);
    let path = query_cache_path(&library);

    // The cache is a build input a checkout with no database validates from,
    // so it lives beside the package and not under the ignored `.dart_tool/`.
    assert_eq!(
        path.parent().unwrap(),
        Path::new(&library.package_root).join(".dust_sql")
    );
    assert!(
        path.extension()
            .is_some_and(|extension| extension == "json")
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn cached_column_validation_reports_missing_row_columns() {
    let mut row_columns = HashMap::new();
    row_columns.insert(
        "UserRow".to_owned(),
        HashSet::from(["id".to_owned(), "email".to_owned()]),
    );

    let error = validate_cached_columns(
        &query(QueryFunction::As, FetchMode::One),
        &row_columns,
        &["id".to_owned()],
    )
    .unwrap_err();

    assert_eq!(
        error,
        "cached SQL metadata for `test.query` does not return required column `email` for row `UserRow`"
    );
}
