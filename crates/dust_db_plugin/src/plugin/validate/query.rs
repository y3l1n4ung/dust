use std::collections::{HashMap, HashSet};

use dust_dart_emit::DART_ROW;
use dust_diagnostics::{Diagnostic, SourceLabel};

use crate::plugin::{
    model::{FetchMode, QueryFunction, QuerySpec},
    sql::{PlaceholderRewrite, rewrite_sqlite_placeholders},
};

use super::types::is_supported_scalar_type;

/// Validates SQL source, parameters, placeholders, and fetch shape.
pub(super) fn validate_query_shape(query: &QuerySpec, diagnostics: &mut Vec<Diagnostic>) {
    if !query.sql_source_static {
        diagnostics.push(query_error(
            query,
            "Database query SQL must be a static string literal",
        ));
        return;
    }
    if !query.params_source_is_list {
        diagnostics.push(query_error(
            query,
            "Database query parameters must be a List literal in v1",
        ));
    }
    if let Err(error) = validate_placeholders(&query.sql, query.parameter_count) {
        diagnostics.push(query_error(query, error));
    }
    match query.function {
        QueryFunction::As => validate_query_as(query, diagnostics),
        QueryFunction::Scalar => validate_query_scalar(query, diagnostics),
        QueryFunction::Raw if query.fetch != FetchMode::Raw => {
            diagnostics.push(query_error(query, "queryRaw must end with fetch"))
        }
        QueryFunction::Execute if query.fetch != FetchMode::Execute => {
            diagnostics.push(query_error(query, "queryExecute must end with execute"))
        }
        QueryFunction::Raw | QueryFunction::Execute => {}
    }
}

/// Validates and rewrites SQLx-style placeholders.
pub(super) fn validate_placeholders(
    sql: &str,
    user_parameter_count: usize,
) -> Result<PlaceholderRewrite, String> {
    rewrite_sqlite_placeholders(sql, user_parameter_count)
}

/// Reports a `queryAs<T>` whose row type has no generated row mapping.
///
/// Without one the call falls through to `RowMapperRegistry`, which resolves at
/// runtime and throws `SqlxError.decode` on the request that reaches it. The
/// row classes resolve package-wide, so this can name the type exactly rather
/// than guess from the file it happens to be looking at.
pub(super) fn validate_row_type_is_mapped(
    query: &QuerySpec,
    row_columns: &HashMap<String, HashSet<String>>,
    ambiguous: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // A call that brings its own mapper needs nothing generated.
    if query.has_row_mapper_argument {
        return;
    }
    let Some(row_type) = query_row_type(query) else {
        return;
    };
    // `queryAs<Row>` asks for the row itself, which needs no mapping. An
    // ambiguous name has its own diagnostic; adding this one would be noise.
    if row_type == DART_ROW || row_columns.contains_key(row_type) || ambiguous.contains(row_type) {
        return;
    }
    diagnostics.push(query_error(
        query,
        format!(
            "queryAs<{row_type}> row type has no row mapping. Add \
             `@Derive([FromRow()])` to `{row_type}`, or pass one with \
             `mapper:` or `using:`"
        ),
    ));
}

/// Returns the row type for `queryAs<T>` specs.
pub(super) fn query_row_type(query: &QuerySpec) -> Option<&str> {
    matches!(query.function, QueryFunction::As).then(|| query.row_type.as_deref())?
}

/// Builds a source-labelled diagnostic for a query spec.
pub(super) fn query_error(query: &QuerySpec, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(message.into()).with_label(SourceLabel::new(
        query.span.file_id,
        query.span.range,
        "invalid Database query",
    ))
}

/// Validates `queryAs<T>` fetch shape and type argument.
fn validate_query_as(query: &QuerySpec, diagnostics: &mut Vec<Diagnostic>) {
    if query.row_type.is_none() {
        diagnostics.push(query_error(query, "queryAs<T> must specify a row type"));
        return;
    }
    if !matches!(
        query.fetch,
        FetchMode::One | FetchMode::Optional | FetchMode::All
    ) {
        diagnostics.push(query_error(
            query,
            "queryAs<T> must end with fetchOne, fetchOptional, or fetchAll",
        ));
    }
}

/// Validates `queryScalar<T>` fetch shape and scalar type.
fn validate_query_scalar(query: &QuerySpec, diagnostics: &mut Vec<Diagnostic>) {
    if query
        .scalar_type
        .as_ref()
        .is_none_or(|ty| !is_supported_scalar_type(ty))
    {
        diagnostics.push(query_error(
            query,
            "queryScalar<T> must use a supported scalar type",
        ));
    }
    if !matches!(query.fetch, FetchMode::One | FetchMode::Optional) {
        diagnostics.push(query_error(
            query,
            "queryScalar<T> must end with fetchOne or fetchOptional",
        ));
    }
}
