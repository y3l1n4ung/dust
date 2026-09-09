//! Rendering the body of one DAO method for each query shape.

use super::*;

/// Rendered SQL literal and bind argument list for generated DAO code.
pub(super) struct RenderedQuery {
    /// Rendered SQL string literal.
    pub(super) sql: String,
    /// Comma-separated method parameter names in bind order.
    pub(super) args: String,
}

/// Renders a generated Dart method parameter list.
pub(super) fn render_method_params(method: &MethodIr) -> String {
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
pub(super) fn render_dao_method_body(
    method: &DaoMethod<'_>,
    row_names: &HashSet<&str>,
    sql: &str,
    args: &str,
) -> String {
    let Some(ok_type) = method.return_ok_type.as_ref() else {
        return render_template(
            "dao_body_invalid_return",
            include_str!("../templates/dao_body_invalid_return.jinja"),
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
            include_str!("../templates/dao_body_scalar.jinja"),
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
        return render_unsupported_body(ok_type, "Unsupported DAO return type.");
    };
    if ok_type.is_nullable() {
        return render_template(
            "dao_body_fetch_optional",
            include_str!("../templates/dao_body_fetch_optional.jinja"),
            RowBodyContext {
                row_name,
                sql,
                args,
            },
        );
    }
    render_template(
        "dao_body_fetch_one",
        include_str!("../templates/dao_body_fetch_one.jinja"),
        RowBodyContext {
            row_name,
            sql,
            args,
        },
    )
}

/// Renders the generated body for `List<T>` DAO returns.
pub(super) fn render_list_body(
    ok_type: &dust_ir::TypeIr,
    row_names: &HashSet<&str>,
    sql: &str,
    args: &str,
) -> String {
    // `List<Row>` and a bare `List` both used to generate an unchecked
    // `_db.raw.fetch`. A DAO holds an executor, and unchecked SQL is reachable
    // only from the database facade now, so there is nothing to generate.
    let Some(item) = ok_type.args().first() else {
        return render_unsupported_body(ok_type, UNTYPED_ROWS_MESSAGE);
    };
    if item.is_named(DART_ROW) {
        return render_unsupported_body(ok_type, UNTYPED_ROWS_MESSAGE);
    }
    let item_name = item.name().unwrap_or(DART_OBJECT);
    if row_names.contains(item_name) {
        return render_template(
            "dao_body_fetch_all",
            include_str!("../templates/dao_body_fetch_all.jinja"),
            ListBodyContext {
                item_name,
                sql,
                args,
            },
        );
    }
    render_unsupported_body(ok_type, "Unsupported DAO list item type.")
}

/// Message for a DAO method asking for rows no row type describes.
pub(super) const UNTYPED_ROWS_MESSAGE: &str = "A DAO cannot return untyped rows. Use a row type with @Derive([FromRow()]), or the unsafe escape hatch on the database facade.";

/// Renders the generated body that reports an unsupported DAO return type.
pub(super) fn render_unsupported_body(ok_type: &dust_ir::TypeIr, message: &str) -> String {
    render_template(
        "dao_body_unsupported",
        include_str!("../templates/dao_body_unsupported.jinja"),
        UnsupportedBodyContext {
            ty: DYNAMIC_TYPES.render(ok_type),
            // The message lands inside a single-quoted Dart string, so an
            // apostrophe in it would emit code that does not parse.
            message: escape_dart_string(message),
        },
    )
}

/// Renders a simple query body template selected by name.
pub(super) fn render_query_body(
    name: &str,
    template: &'static str,
    sql: &str,
    args: &str,
) -> String {
    let source = match template {
        "templates/dao_body_execute.jinja" => include_str!("../templates/dao_body_execute.jinja"),
        "templates/dao_body_unit.jinja" => include_str!("../templates/dao_body_unit.jinja"),
        _ => unreachable!("unknown DAO query body template"),
    };
    render_template(name, source, QueryBodyContext { sql, args })
}
