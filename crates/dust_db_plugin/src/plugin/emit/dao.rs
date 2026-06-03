use std::collections::HashSet;

use dust_dart_emit::{DYNAMIC_TYPES, render_template};
use dust_ir::{MethodIr, ParamKind};
use serde::Serialize;

use crate::plugin::{
    model::{DaoClass, DaoMethod, DbDriver},
    sql::rewrite_sqlite_placeholders,
};

use super::shared::{is_scalar_type, render_sql_literal};

#[derive(Serialize)]
struct DaoClassContext<'a> {
    generated_name: &'a str,
    class_name: &'a str,
    methods: String,
}

#[derive(Serialize)]
struct DaoMethodContext<'a> {
    return_type: String,
    method_name: &'a str,
    params: String,
    body: String,
}

#[derive(Serialize)]
struct InvalidReturnContext<'a> {
    method_name: &'a str,
}

#[derive(Serialize)]
struct QueryBodyContext<'a> {
    sql: &'a str,
    args: &'a str,
}

#[derive(Serialize)]
struct ScalarBodyContext<'a> {
    ty: String,
    sql: &'a str,
    args: &'a str,
}

#[derive(Serialize)]
struct RowBodyContext<'a> {
    row_name: &'a str,
    sql: &'a str,
    args: &'a str,
}

#[derive(Serialize)]
struct ListBodyContext<'a> {
    item_name: &'a str,
    sql: &'a str,
    args: &'a str,
}

#[derive(Serialize)]
struct UnsupportedBodyContext {
    ty: String,
    message: &'static str,
}

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

struct RenderedQuery {
    sql: String,
    args: String,
}

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
    if ok_type.is_named("ExecResult") {
        return render_query_body(
            "dao_body_execute",
            "templates/dao_body_execute.jinja",
            sql,
            args,
        );
    }
    if ok_type.is_named("Unit") {
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
    if ok_type.is_named("List") {
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
    if item.is_named("Row") {
        return render_query_body(
            "dao_body_raw_fetch",
            "templates/dao_body_raw_fetch.jinja",
            sql,
            args,
        );
    }
    let item_name = item.name().unwrap_or("Object");
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

fn render_query_body(name: &str, template: &'static str, sql: &str, args: &str) -> String {
    let source = match template {
        "templates/dao_body_execute.jinja" => include_str!("templates/dao_body_execute.jinja"),
        "templates/dao_body_unit.jinja" => include_str!("templates/dao_body_unit.jinja"),
        "templates/dao_body_raw_fetch.jinja" => include_str!("templates/dao_body_raw_fetch.jinja"),
        _ => unreachable!("unknown DAO query body template"),
    };
    render_template(name, source, QueryBodyContext { sql, args })
}
