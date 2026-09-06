use dust_driver::{CheckRequest, DbRequestOptions, run_check};

use super::helpers::{
    assert_sqlx_rejected, assert_static_sql_rejected, write_query_validation_workspace,
    write_static_sql_validation_workspace,
};
use crate::support::make_workspace;

#[test]
fn db_check_rejects_sql_variable() {
    assert_static_sql_rejected(
        "final sql = 'SELECT id FROM users';\n\
         return queryExecute(sql, []).execute(this);",
    );
}

#[test]
fn db_check_rejects_const_sql_variable() {
    assert_static_sql_rejected(
        "const sql = 'SELECT id FROM users';\n\
         return queryExecute(sql, []).execute(this);",
    );
}

#[test]
fn db_check_rejects_interpolated_sql_literal() {
    assert_static_sql_rejected(
        "const table = 'users';\n\
         return queryExecute('SELECT id FROM $table', []).execute(this);",
    );
}

#[test]
fn db_check_rejects_concatenated_sql_literals() {
    assert_static_sql_rejected("return queryExecute('SELECT id ' 'FROM users', []).execute(this);");
}

#[test]
fn db_check_rejects_non_list_query_parameters() {
    let workspace = make_workspace();
    write_static_sql_validation_workspace(
        workspace.path(),
        "return queryExecute('SELECT id FROM users WHERE id = 1', params).execute(this);",
    );

    let result = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });

    assert!(result.has_errors());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("query parameters must be a List literal")),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn db_check_accepts_runtime_values_inside_parameter_list_literal() {
    let workspace = make_workspace();
    write_static_sql_validation_workspace(
        workspace.path(),
        "return queryExecute(r'SELECT id FROM users WHERE id = $1', [params.first]).execute(this);",
    );

    let result = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
}

#[test]
fn db_check_accepts_raw_multiline_sql_literal() {
    let workspace = make_workspace();
    write_static_sql_validation_workspace(
        workspace.path(),
        "return queryExecute(r'''\nSELECT id\nFROM users\nWHERE id = $1\n''', [params.first]).execute(this);",
    );

    let result = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
}

#[test]
fn db_check_rejects_unknown_table_via_sqlx() {
    assert_sqlx_rejected(
        "return queryExecute('SELECT id FROM missing_users', []).execute(this);",
        "SQLx rejected",
    );
}

#[test]
fn db_check_rejects_unknown_column_via_sqlx() {
    assert_sqlx_rejected(
        "return queryExecute('SELECT missing_id FROM users', []).execute(this);",
        "SQLx rejected",
    );
}

#[test]
fn db_check_rejects_scalar_query_returning_multiple_columns() {
    assert_sqlx_rejected(
        "return queryScalar<int>('SELECT id, name FROM users', []).fetchOne(this);",
        "must return exactly one scalar column",
    );
}

#[test]
fn db_check_rejects_from_row_query_missing_required_column() {
    let workspace = make_workspace();
    write_query_validation_workspace(
        workspace.path(),
        "Future<List<UserRow>> allUsers() {\n\
           return queryAs<UserRow>('SELECT id FROM users', []).fetchAll(this);\n\
         }",
    );

    let result = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: DbRequestOptions {
            only_db: true,
            offline: false,
        },
    });

    assert!(result.has_errors());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("does not return required column `name` for row `UserRow`")),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn db_check_rejects_placeholder_count_mismatch_before_runtime() {
    assert_sqlx_rejected(
        "return queryExecute(r'SELECT id FROM users WHERE id = $1', []).execute(this);",
        "query binds 0 args but SQL expects 1 parameters",
    );
}
