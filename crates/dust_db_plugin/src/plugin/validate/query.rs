use dust_diagnostics::{Diagnostic, SourceLabel};

use crate::plugin::model::{FetchMode, QueryFunction, QuerySpec};

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
    let mut rewritten = String::new();
    let mut order = Vec::<usize>::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let bytes = sql.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let ch = sql[i..].chars().next().unwrap_or_default();
        if ch == '\'' && !in_double_quote {
            rewritten.push(ch);
            if in_single_quote && sql[i + ch.len_utf8()..].starts_with('\'') {
                i += ch.len_utf8();
                rewritten.push('\'');
            } else {
                in_single_quote = !in_single_quote;
            }
            i += ch.len_utf8();
            continue;
        }
        if ch == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            rewritten.push(ch);
            i += ch.len_utf8();
            continue;
        }
        if ch == '$' && !in_single_quote && !in_double_quote {
            let mut end = i + 1;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end > i + 1 {
                let index = sql[i + 1..end]
                    .parse::<usize>()
                    .map_err(|_| "invalid SQL placeholder".to_owned())?;
                if index == 0 {
                    return Err("SQL placeholders are 1-based".to_owned());
                }
                order.push(index);
                rewritten.push('?');
                i = end;
                continue;
            }
        }
        rewritten.push(ch);
        i += ch.len_utf8();
    }

    let max = order.iter().copied().max().unwrap_or(0);
    for index in 1..=max {
        if !order.contains(&index) {
            return Err(format!("SQL placeholders must not skip `${index}`"));
        }
    }
    if user_parameter_count != max {
        return Err(format!(
            "query binds {user_parameter_count} args but SQL expects {max} parameters"
        ));
    }
    Ok(PlaceholderRewrite {
        sql: rewritten,
        expanded_parameter_count: order.len(),
    })
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

#[derive(Debug)]
pub(super) struct PlaceholderRewrite {
    pub(super) sql: String,
    pub(super) expanded_parameter_count: usize,
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
