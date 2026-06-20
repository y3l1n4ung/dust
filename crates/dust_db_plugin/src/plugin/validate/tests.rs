use std::{
    collections::{HashMap, HashSet},
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use dust_ir::{DartFileIr, SpanIr, TypeIr};
use dust_text::{FileId, TextRange};

use super::{
    cache::{
        QUERY_CACHE_VERSION, QueryCache, QueryCacheEntry, migration_files, query_cache_path,
        schema_hash, validate_cached_columns, validate_from_query_cache,
    },
    query::{validate_placeholders, validate_query_shape},
};
use crate::plugin::model::{FetchMode, QueryFunction, QuerySpec};

/// Builds a small source span for validation test fixtures.
fn span() -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(0_u32, 1_u32))
}

/// Builds a query fixture with common valid defaults.
fn query(function: QueryFunction, fetch: FetchMode) -> QuerySpec {
    QuerySpec {
        function,
        fetch,
        sql: "SELECT id FROM users WHERE id = $1".to_owned(),
        sql_source_static: true,
        row_type: (function == QueryFunction::As).then(|| "UserRow".to_owned()),
        scalar_type: (function == QueryFunction::Scalar).then(TypeIr::int),
        parameter_count: 1,
        params_source_is_list: true,
        span: span(),
        display_name: Some("test.query".to_owned()),
    }
}

/// Builds a unique temporary package root for validation tests.
fn temp_root(name: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("dust_db_validate_{name}_{stamp}"))
}

/// Builds a minimal Dart library fixture rooted at a temporary package.
fn library(root: &std::path::Path) -> DartFileIr {
    DartFileIr {
        package_root: root.display().to_string(),
        package_name: "example".to_owned(),
        source_path: "lib/db.dart".to_owned(),
        output_path: "lib/db.g.dart".to_owned(),
        imports: Vec::new(),
        library: None,
        library_annotations: Vec::new(),
        import_directives: Vec::new(),
        export_directives: Vec::new(),
        part_directives: Vec::new(),
        part_of: None,
        span: span(),
        classes: Vec::new(),
        mixins: Vec::new(),
        extensions: Vec::new(),
        extension_types: Vec::new(),
        functions: Vec::new(),
        variables: Vec::new(),
        typedefs: Vec::new(),
        enums: Vec::new(),
        query_calls: Vec::new(),
    }
}

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
    assert_eq!(rewrite.parameter_order, vec![1, 1, 2]);
    assert_eq!(rewrite.expanded_parameter_count(), 3);
}

#[test]
fn placeholder_validation_rejects_zero_gaps_and_count_mismatch() {
    assert_eq!(
        validate_placeholders("SELECT * FROM users WHERE id = $0", 1).unwrap_err(),
        "SQL placeholders are 1-based"
    );
    assert_eq!(
        validate_placeholders("SELECT * FROM users WHERE id = $2", 2).unwrap_err(),
        "SQL placeholders must not skip `$1`"
    );
    assert_eq!(
        validate_placeholders("SELECT * FROM users WHERE id = $1", 0).unwrap_err(),
        "query binds 0 args but SQL expects 1 parameters"
    );
}

#[test]
fn placeholder_validation_handles_quotes_and_escaped_single_quotes() {
    let rewrite = validate_placeholders(
        "SELECT '$1', 'it''s $2', \"$3\" FROM users WHERE id = $1",
        1,
    )
    .unwrap();

    assert_eq!(
        rewrite.sql,
        "SELECT '$1', 'it''s $2', \"$3\" FROM users WHERE id = ?"
    );
    assert_eq!(rewrite.parameter_order, vec![1]);
    assert_eq!(rewrite.expanded_parameter_count(), 1);
}

#[test]
fn query_shape_validation_rejects_invalid_fetch_shapes() {
    let mut diagnostics = Vec::new();
    validate_query_shape(&query(QueryFunction::As, FetchMode::Raw), &mut diagnostics);
    validate_query_shape(
        &QuerySpec {
            row_type: None,
            ..query(QueryFunction::As, FetchMode::One)
        },
        &mut diagnostics,
    );
    validate_query_shape(
        &QuerySpec {
            scalar_type: Some(TypeIr::named("Object")),
            ..query(QueryFunction::Scalar, FetchMode::One)
        },
        &mut diagnostics,
    );
    validate_query_shape(&query(QueryFunction::Raw, FetchMode::One), &mut diagnostics);
    validate_query_shape(
        &query(QueryFunction::Execute, FetchMode::Raw),
        &mut diagnostics,
    );

    let messages = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec![
            "queryAs<T> must end with fetchOne, fetchOptional, or fetchAll",
            "queryAs<T> must specify a row type",
            "queryScalar<T> must use a supported scalar type",
            "queryRaw must end with fetch",
            "queryExecute must end with execute",
        ]
    );
}

#[test]
fn query_shape_validation_accepts_valid_shapes() {
    let mut diagnostics = Vec::new();
    validate_query_shape(&query(QueryFunction::As, FetchMode::One), &mut diagnostics);
    validate_query_shape(
        &query(QueryFunction::As, FetchMode::Optional),
        &mut diagnostics,
    );
    validate_query_shape(&query(QueryFunction::As, FetchMode::All), &mut diagnostics);
    validate_query_shape(
        &query(QueryFunction::Scalar, FetchMode::One),
        &mut diagnostics,
    );
    validate_query_shape(
        &query(QueryFunction::Execute, FetchMode::Execute),
        &mut diagnostics,
    );

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn query_shape_validation_rejects_non_static_sql_and_non_list_params() {
    let mut diagnostics = Vec::new();
    validate_query_shape(
        &QuerySpec {
            sql_source_static: false,
            ..query(QueryFunction::Raw, FetchMode::Raw)
        },
        &mut diagnostics,
    );
    validate_query_shape(
        &QuerySpec {
            params_source_is_list: false,
            ..query(QueryFunction::Raw, FetchMode::Raw)
        },
        &mut diagnostics,
    );

    let messages = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec![
            "Dust DB query SQL must be a static string literal",
            "Dust DB query parameters must be a List literal in v1",
        ]
    );
}

#[test]
fn migration_files_are_sorted_and_schema_hash_is_stable() {
    let root = temp_root("migrations");
    fs::create_dir_all(root.join("migrations")).unwrap();
    fs::write(root.join("migrations/002_second.sql"), "SELECT 2;\n").unwrap();
    fs::write(root.join("migrations/001_first.sql"), "SELECT 1;\n").unwrap();

    let files = migration_files(&root.join("migrations")).unwrap();
    let names = files
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["001_first.sql", "002_second.sql"]);
    assert_eq!(
        schema_hash(&root.join("migrations")).unwrap(),
        schema_hash(&root.join("migrations")).unwrap()
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
            migrations: "./migrations".to_owned(),
            schema_hash: "schema".to_owned(),
            sql_hash: super::cache::stable_hash_hex(query.sql.as_bytes()),
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
            "./migrations",
            "schema",
            std::slice::from_ref(&query),
            &row_columns,
        ),
        Ok(())
    );
    assert!(
        validate_from_query_cache(&library, "./migrations", "stale", &[query], &row_columns)
            .unwrap_err()
            .contains("missing entry")
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
