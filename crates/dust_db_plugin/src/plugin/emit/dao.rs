use std::collections::HashSet;

use dust_dart_emit::{
    DART_EXEC_RESULT, DART_LIST, DART_OBJECT, DART_ROW, DART_UNIT, DYNAMIC_TYPES, render_template,
};
use dust_ir::{MethodIr, ParamKind};
use serde::Serialize;

use crate::plugin::{
    model::{DaoClass, DaoMethod, DbDriver},
    sql::rewrite_sqlite_placeholders,
};

use super::shared::{is_scalar_type, render_sql_literal};

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
    /// Static error message emitted in generated code.
    message: &'static str,
}

/// Renders a generated DAO implementation class.
pub(super) fn render_dao_class(
    dao: &DaoClass<'_>,
    row_names: &HashSet<&str>,
    driver: DbDriver,
) -> String {
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
        .map(|method| render_dao_method(method, row_names, driver))
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
fn render_dao_method(
    method: &DaoMethod<'_>,
    row_names: &HashSet<&str>,
    driver: DbDriver,
) -> String {
    let method_ir = method.method;
    let return_type = DYNAMIC_TYPES.render(&method_ir.return_type);
    let params = render_method_params(method_ir);
    let rendered_query = render_driver_query(method, driver);
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

/// Renders SQL and argument order for the target database driver.
fn render_driver_query(method: &DaoMethod<'_>, driver: DbDriver) -> RenderedQuery {
    let params = method.method.params.as_slice();
    if matches!(driver, DbDriver::Sqlite3) {
        if let Ok(rewrite) = rewrite_sqlite_placeholders(&method.sql, params.len()) {
            let args = rewrite
                .parameter_order
                .iter()
                .filter_map(|index| params.get(index.saturating_sub(1)))
                .map(|param| param.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            return RenderedQuery {
                sql: render_sql_literal(&rewrite.sql),
                args,
            };
        }
    }

    RenderedQuery {
        sql: render_sql_literal(&method.sql),
        args: params
            .iter()
            .map(|param| param.name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// Rendered SQL literal and bind argument list for generated DAO code.
struct RenderedQuery {
    /// Rendered SQL string literal.
    sql: String,
    /// Comma-separated method parameter names in bind order.
    args: String,
}

/// Renders a generated Dart method parameter list.
fn render_method_params(method: &MethodIr) -> String {
    let mut positional = Vec::new();
    let mut named = Vec::new();
    for param in &method.params {
        let rendered = format!("{} {}", DYNAMIC_TYPES.render(&param.ty), param.name);
        match param.kind {
            ParamKind::Positional => positional.push(rendered),
            ParamKind::Named => named.push(rendered),
        }
    }
    if named.is_empty() {
        positional.join(", ")
    } else {
        let mut parts = positional;
        parts.push(format!("{{{}}}", named.join(", ")));
        parts.join(", ")
    }
}

/// Renders the generated body for a DAO method based on its return type.
fn render_dao_method_body(
    method: &DaoMethod<'_>,
    row_names: &HashSet<&str>,
    sql: &str,
    args: &str,
) -> String {
    let Some(ok_type) = method.return_ok_type.as_ref() else {
        return render_template(
            "dao_body_invalid_return",
            include_str!("templates/dao_body_invalid_return.jinja"),
            InvalidReturnContext {
                method_name: &method.method.name,
            },
        );
    };
    if ok_type.is_named(DART_EXEC_RESULT) {
        return render_query_body(
            "dao_body_execute",
            "templates/dao_body_execute.jinja",
            sql,
            args,
        );
    }
    if ok_type.is_named(DART_UNIT) {
        return render_query_body("dao_body_unit", "templates/dao_body_unit.jinja", sql, args);
    }
    if is_scalar_type(ok_type) {
        return render_template(
            "dao_body_scalar",
            include_str!("templates/dao_body_scalar.jinja"),
            ScalarBodyContext {
                ty: DYNAMIC_TYPES.render(ok_type),
                sql,
                args,
            },
        );
    }
    if ok_type.is_named(DART_LIST) {
        return render_list_body(ok_type, row_names, sql, args);
    }
    let Some(row_name) = ok_type.name() else {
        return render_template(
            "dao_body_unsupported",
            include_str!("templates/dao_body_unsupported.jinja"),
            UnsupportedBodyContext {
                ty: DYNAMIC_TYPES.render(ok_type),
                message: "Unsupported DAO return type.",
            },
        );
    };
    if ok_type.is_nullable() {
        return render_template(
            "dao_body_fetch_optional",
            include_str!("templates/dao_body_fetch_optional.jinja"),
            RowBodyContext {
                row_name,
                sql,
                args,
            },
        );
    }
    render_template(
        "dao_body_fetch_one",
        include_str!("templates/dao_body_fetch_one.jinja"),
        RowBodyContext {
            row_name,
            sql,
            args,
        },
    )
}

/// Renders the generated body for `List<T>` DAO returns.
fn render_list_body(
    ok_type: &dust_ir::TypeIr,
    row_names: &HashSet<&str>,
    sql: &str,
    args: &str,
) -> String {
    let Some(item) = ok_type.args().first() else {
        return render_query_body(
            "dao_body_raw_fetch",
            "templates/dao_body_raw_fetch.jinja",
            sql,
            args,
        );
    };
    if item.is_named(DART_ROW) {
        return render_query_body(
            "dao_body_raw_fetch",
            "templates/dao_body_raw_fetch.jinja",
            sql,
            args,
        );
    }
    let item_name = item.name().unwrap_or(DART_OBJECT);
    if row_names.contains(item_name) {
        return render_template(
            "dao_body_fetch_all",
            include_str!("templates/dao_body_fetch_all.jinja"),
            ListBodyContext {
                item_name,
                sql,
                args,
            },
        );
    }
    render_template(
        "dao_body_unsupported",
        include_str!("templates/dao_body_unsupported.jinja"),
        UnsupportedBodyContext {
            ty: DYNAMIC_TYPES.render(ok_type),
            message: "Unsupported DAO list item type.",
        },
    )
}

/// Renders a simple query body template selected by name.
fn render_query_body(name: &str, template: &'static str, sql: &str, args: &str) -> String {
    let source = match template {
        "templates/dao_body_execute.jinja" => include_str!("templates/dao_body_execute.jinja"),
        "templates/dao_body_unit.jinja" => include_str!("templates/dao_body_unit.jinja"),
        "templates/dao_body_raw_fetch.jinja" => include_str!("templates/dao_body_raw_fetch.jinja"),
        _ => unreachable!("unknown DAO query body template"),
    };
    render_template(name, source, QueryBodyContext { sql, args })
}
