use std::collections::HashSet;

use dust_dart_emit::DYNAMIC_TYPES;
use dust_ir::{MethodIr, ParamKind};

use crate::plugin::{
    model::{DaoClass, DaoMethod, DbDriver},
    sql::rewrite_sqlite_placeholders,
};

use super::shared::{is_scalar_type, render_sql_literal};

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

    format!(
        "final class {generated_name} implements {class_name} {{\n  const {generated_name}(this._db);\n\n  final SqlxDriver _db;\n\n{methods}\n}}"
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

    format!(
        "  @override\n  {return_type} {}({params}) {{\n{body}\n  }}",
        method_ir.name
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
        return format!(
            "    return Err<dynamic, SqlxError>(\n      SqlxError.decode('DAO method `{}` must return Future<Result<T, SqlxError>>.'),\n    );",
            method.method.name
        );
    };
    if ok_type.is_named("ExecResult") {
        return format!("    return _db.execute(\n      {sql},\n      [{args}],\n    );");
    }
    if ok_type.is_named("Unit") {
        return format!(
            "    return _db.execute(\n      {sql},\n      [{args}],\n    ).then(\n      (result) => result.andThen<Unit>((_) => const Ok<Unit, SqlxError>(unit)),\n    );"
        );
    }
    if is_scalar_type(ok_type) {
        return format!(
            "    return _db.fetchScalar<{}>(\n      {sql},\n      [{args}],\n    );",
            DYNAMIC_TYPES.render(ok_type)
        );
    }
    if ok_type.is_named("List") {
        return render_list_body(ok_type, row_names, sql, args);
    }
    let Some(row_name) = ok_type.name() else {
        return format!(
            "      return Err<{}, SqlxError>(\n        SqlxError.decode('Unsupported DAO return type.'),\n      );",
            DYNAMIC_TYPES.render(ok_type)
        );
    };
    if ok_type.is_nullable() {
        return format!(
            "    return _db.fetchOptional<{row_name}>(\n      {sql},\n      [{args}],\n      {row_name}FromRow.fromRow,\n    );"
        );
    }
    format!(
        "    return _db.fetchOne<{row_name}>(\n      {sql},\n      [{args}],\n      {row_name}FromRow.fromRow,\n    );"
    )
}

fn render_list_body(
    ok_type: &dust_ir::TypeIr,
    row_names: &HashSet<&str>,
    sql: &str,
    args: &str,
) -> String {
    let Some(item) = ok_type.args().first() else {
        return format!("    return _db.raw.fetch(\n      {sql},\n      [{args}],\n    );");
    };
    if item.is_named("Row") {
        return format!("    return _db.raw.fetch(\n      {sql},\n      [{args}],\n    );");
    }
    let item_name = item.name().unwrap_or("Object");
    if row_names.contains(item_name) {
        return format!(
            "    return _db.fetchAll<{item_name}>(\n      {sql},\n      [{args}],\n      {item_name}FromRow.fromRow,\n    );"
        );
    }
    format!(
        "    return Err<{}, SqlxError>(\n      SqlxError.decode('Unsupported DAO list item type.'),\n    );",
        DYNAMIC_TYPES.render(ok_type)
    )
}
