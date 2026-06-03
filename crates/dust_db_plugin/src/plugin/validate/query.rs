use dust_diagnostics::{Diagnostic, SourceLabel};

use crate::plugin::{
    model::{FetchMode, QueryFunction, QuerySpec},
    sql::{PlaceholderRewrite, rewrite_sqlite_placeholders},
};

use super::types::is_supported_scalar_type;

pub(super) fn validate_query_shape(query: &QuerySpec, diagnostics: &mut Vec<Diagnostic>) {
    if !query.sql_source_static {
        diagnostics.push(query_error(
            query,
            "Dust DB query SQL must be a static string literal",
        ));
        return;
    }
    if !query.params_source_is_list {
        diagnostics.push(query_error(
            query,
            "Dust DB query parameters must be a List literal in v1",
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

pub(super) fn validate_placeholders(
    sql: &str,
    user_parameter_count: usize,
) -> Result<PlaceholderRewrite, String> {
    rewrite_sqlite_placeholders(sql, user_parameter_count)
}

pub(super) fn query_row_type(query: &QuerySpec) -> Option<&str> {
    matches!(query.function, QueryFunction::As).then(|| query.row_type.as_deref())?
}

pub(super) fn query_error(query: &QuerySpec, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(message.into()).with_label(SourceLabel::new(
        query.span.file_id,
        query.span.range,
        "invalid Dust DB query",
    ))
}

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
