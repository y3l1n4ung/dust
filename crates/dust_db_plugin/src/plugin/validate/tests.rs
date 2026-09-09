use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use dust_diagnostics::Severity;
use dust_ir::{DartFileIr, SpanIr, TypeIr};
use dust_text::{FileId, TextRange};

use super::{
    cache::{
        QUERY_CACHE_VERSION, QueryCache, QueryCacheEntry, query_cache_path,
        validate_cached_columns, validate_from_query_cache,
    },
    hash::schema_hash,
    query::{validate_placeholders, validate_query_shape},
};
use crate::plugin::{
    migrations::applied_migration_files,
    model::DbDriver,
    model::{FetchMode, QueryFunction, QuerySpec},
};

/// Migration ordering, reversible pairs, and the offline query cache.
#[path = "tests/migrations.rs"]
mod migrations;

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
        has_row_mapper_argument: false,
        unsafe_sql_allowed: false,
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
    validate_query_shape(
        &query(QueryFunction::As, FetchMode::Execute),
        &mut diagnostics,
    );
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
    validate_query_shape(
        &query(QueryFunction::Unsupported, FetchMode::Unsupported),
        &mut diagnostics,
    );
    validate_query_shape(
        &query(QueryFunction::Execute, FetchMode::One),
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
            "Database query has an unsupported return type. Return `Future<Result<T, SqlxError>>` for a row type, a supported scalar, `ExecResult`, or `Unit`. Untyped rows come from the database facade's `unsafe` escape hatch, not from a DAO",
            "queryExecute must end with execute",
        ]
    );
}

#[test]
fn unchecked_sql_warns_once_per_call_and_skips_every_other_check() {
    let mut diagnostics = Vec::new();
    // Dynamic SQL and a non-list parameter argument are exactly what the escape
    // hatch is for, so neither may be reported against it.
    validate_query_shape(
        &QuerySpec {
            sql_source_static: false,
            params_source_is_list: false,
            ..query(QueryFunction::Unsafe, FetchMode::Unsupported)
        },
        &mut diagnostics,
    );

    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].severity, Severity::Warning);
    assert!(
        diagnostics[0]
            .message
            .contains("unchecked SQL bypasses build-time validation"),
        "{diagnostics:?}"
    );
    assert!(
        diagnostics[0]
            .notes
            .iter()
            .any(|note| note.contains("dust:allow-unsafe-sql")),
        "{diagnostics:?}"
    );
}

#[test]
fn a_marker_comment_silences_the_unchecked_sql_warning() {
    let mut diagnostics = Vec::new();
    validate_query_shape(
        &QuerySpec {
            unsafe_sql_allowed: true,
            sql_source_static: false,
            ..query(QueryFunction::Unsafe, FetchMode::Unsupported)
        },
        &mut diagnostics,
    );

    assert_eq!(diagnostics, Vec::new());
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
            ..query(QueryFunction::Execute, FetchMode::Execute)
        },
        &mut diagnostics,
    );
    validate_query_shape(
        &QuerySpec {
            params_source_is_list: false,
            ..query(QueryFunction::Execute, FetchMode::Execute)
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
            "Database query SQL must be a static string literal",
            "Database query parameters must be a List literal in v1",
        ]
    );
}
