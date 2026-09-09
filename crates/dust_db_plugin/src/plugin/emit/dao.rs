use std::collections::HashSet;

use dust_dart_emit::{
    DART_EXEC_RESULT, DART_LIST, DART_OBJECT, DART_ROW, DART_UNIT, DYNAMIC_TYPES, render_template,
};
use dust_ir::{MethodIr, ParamKind};
use serde::Serialize;

use crate::plugin::model::{DaoClass, DaoMethod};

use super::shared::{escape_dart_string, is_scalar_type, render_sql_literal};

/// Rendering the body of one DAO method for each query shape.
mod bodies;
use self::bodies::*;

/// Template context for a generated DAO implementation class.
#[derive(Serialize)]
struct DaoClassContext<'a> {
    /// Generated private implementation class name.
    generated_name: &'a str,
    /// Source DAO interface class name.
    class_name: &'a str,
    /// Rendered generated DAO methods.
    methods: String,
}

/// Template context for one generated DAO method.
#[derive(Serialize)]
struct DaoMethodContext<'a> {
    /// Rendered Dart return type.
    return_type: String,
    /// Source method name.
    method_name: &'a str,
    /// Rendered method parameter list.
    params: String,
    /// Rendered method body.
    body: String,
}

/// Template context for unsupported DAO return types.
#[derive(Serialize)]
struct InvalidReturnContext<'a> {
    /// Source method name used in the thrown error.
    method_name: &'a str,
}

/// Template context for untyped query bodies.
#[derive(Serialize)]
struct QueryBodyContext<'a> {
    /// Rendered SQL string literal.
    sql: &'a str,
    /// Rendered argument list.
    args: &'a str,
}

/// Template context for scalar query bodies.
#[derive(Serialize)]
struct ScalarBodyContext<'a> {
    /// Rendered scalar Dart type.
    ty: String,
    /// Rendered SQL string literal.
    sql: &'a str,
    /// Rendered argument list.
    args: &'a str,
}

/// Template context for single-row query bodies.
#[derive(Serialize)]
struct RowBodyContext<'a> {
    /// Row class name used for decoding.
    row_name: &'a str,
    /// Rendered SQL string literal.
    sql: &'a str,
    /// Rendered argument list.
    args: &'a str,
}

/// Template context for list query bodies.
#[derive(Serialize)]
struct ListBodyContext<'a> {
    /// List item row class name.
    item_name: &'a str,
    /// Rendered SQL string literal.
    sql: &'a str,
    /// Rendered argument list.
    args: &'a str,
}

/// Template context for unsupported generated query bodies.
#[derive(Serialize)]
struct UnsupportedBodyContext {
    /// Unsupported Dart type rendered for diagnostics.
    ty: String,
    /// Error message emitted in generated code, escaped for a Dart literal.
    message: String,
}

/// Renders a generated DAO implementation class.
pub(super) fn render_dao_class(dao: &DaoClass<'_>, row_names: &HashSet<&str>) -> String {
    let class_name = &dao.class.name;
    let generated_name = dao
        .class
        .constructors
        .iter()
        .find_map(|constructor| constructor.redirected_target_name.as_deref())
        .filter(|target| target.starts_with("_$"))
        .map_or_else(|| format!("_${class_name}"), str::to_owned);
    let methods = dao
        .methods
        .iter()
        .map(|method| render_dao_method(method, row_names))
        .collect::<Vec<_>>()
        .join("\n\n");

    render_template(
        "dao_class",
        include_str!("templates/dao_class.jinja"),
        DaoClassContext {
            generated_name: &generated_name,
            class_name,
            methods,
        },
    )
}

/// Renders one generated DAO method.
fn render_dao_method(method: &DaoMethod<'_>, row_names: &HashSet<&str>) -> String {
    let method_ir = method.method;
    let return_type = DYNAMIC_TYPES.render(&method_ir.return_type);
    let params = render_method_params(method_ir);
    let rendered_query = render_driver_query(method);
    let body = render_dao_method_body(method, row_names, &rendered_query.sql, &rendered_query.args);

    render_template(
        "dao_method",
        include_str!("templates/dao_method.jinja"),
        DaoMethodContext {
            return_type,
            method_name: &method_ir.name,
            params,
            body,
        },
    )
}

/// Renders the SQL literal and bind arguments for a DAO method.
///
/// The SQL is emitted verbatim, `$n` and all. Which placeholder form the
/// database receives is the driver's business: Postgres takes `$n` unchanged
/// and SQLite rewrites it at bind time, so generated code that picked one would
/// be wrong for the other. It also kept a `@SqlxDao` method and an inline query
/// from agreeing about what the same text means, since only the first was ever
/// rewritten.
fn render_driver_query(method: &DaoMethod<'_>) -> RenderedQuery {
    RenderedQuery {
        sql: render_sql_literal(&method.sql),
        args: method
            .method
            .params
            .iter()
            .map(|param| param.name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
    }
}
