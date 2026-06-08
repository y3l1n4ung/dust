use std::collections::HashSet;

use dust_dart_emit::{DART_EXEC_RESULT, DART_LIST, DART_ROW, DART_UNIT, DYNAMIC_TYPES};
use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_ir::TypeIr;

use crate::plugin::{
    model::{DaoMethod, RowClass},
    parse::{dao_classes, imported_row_names, result_ok_type},
};

use super::types::is_supported_scalar_type;

pub(super) fn validate_daos(
    library: &dust_ir::LibraryIr,
    rows: &[RowClass<'_>],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let imported_rows = imported_row_names(library);
    let row_names = rows
        .iter()
        .map(|row| row.class.name.as_str())
        .chain(imported_rows.iter().map(String::as_str))
        .collect::<HashSet<_>>();
    for dao in dao_classes(library) {
        validate_dao_constructor(&dao, diagnostics);
        for method in dao.methods {
            validate_dao_method(&dao.class.name, &method, &row_names, diagnostics);
        }
    }
}

fn validate_dao_constructor(
    dao: &crate::plugin::model::DaoClass<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let expected_target = format!("_${}", dao.class.name);
    let has_redirecting_factory = dao.class.constructors.iter().any(|constructor| {
        constructor.is_factory
            && constructor.redirected_target_name.as_deref() == Some(expected_target.as_str())
            && constructor.params.len() == 1
            && (constructor.params[0].ty.is_named("Executor")
                || constructor.params[0].ty.is_named("SqlxDriver"))
            && matches!(constructor.params[0].kind, dust_ir::ParamKind::Positional)
    });
    if !has_redirecting_factory {
        diagnostics.push(
            Diagnostic::error(format!(
                "SqlxDao `{}` must declare `const factory {}(Executor db) = _${}`",
                dao.class.name, dao.class.name, dao.class.name
            ))
            .with_label(SourceLabel::new(
                dao.class.span.file_id,
                dao.class.span.range,
                "invalid SQLx DAO constructor",
            )),
        );
    }
}

fn validate_dao_method(
    dao_name: &str,
    method: &DaoMethod<'_>,
    row_names: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    validate_method_body(dao_name, method, diagnostics);
    validate_method_params(dao_name, method, diagnostics);
    if !method.sql_source_static {
        diagnostics.push(
            Diagnostic::error(format!(
                "SqlxDao query method `{}.{}` SQL must be a static raw string literal",
                dao_name, method.method.name
            ))
            .with_label(SourceLabel::new(
                method.method.span.file_id,
                method.method.span.range,
                "invalid @Query SQL",
            )),
        );
    }
    let Some(ok_type) = result_ok_type(&method.method.return_type) else {
        diagnostics.push(
            Diagnostic::error(format!(
                "SqlxDao query method `{}.{}` must return Future<Result<T, SqlxError>>",
                dao_name, method.method.name
            ))
            .with_label(SourceLabel::new(
                method.method.span.file_id,
                method.method.span.range,
                "invalid DAO return type",
            )),
        );
        return;
    };
    if !is_supported_dao_ok_type(ok_type, row_names) {
        diagnostics.push(
            Diagnostic::error(format!(
                "unsupported SqlxDao result type `{}` on `{}.{}`",
                DYNAMIC_TYPES.render(ok_type),
                dao_name,
                method.method.name
            ))
            .with_label(SourceLabel::new(
                method.method.span.file_id,
                method.method.span.range,
                "unsupported DAO result type",
            )),
        );
    }
}

fn validate_method_body(dao_name: &str, method: &DaoMethod<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if method.method.has_body {
        diagnostics.push(
            Diagnostic::error(format!(
                "SqlxDao query method `{}.{}` must be abstract",
                dao_name, method.method.name
            ))
            .with_label(SourceLabel::new(
                method.method.span.file_id,
                method.method.span.range,
                "remove the method body from generated DAO queries",
            )),
        );
    }
}

fn validate_method_params(
    dao_name: &str,
    method: &DaoMethod<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for param in &method.method.params {
        if !matches!(param.kind, dust_ir::ParamKind::Positional) || param.has_default {
            diagnostics.push(
                Diagnostic::error(format!(
                    "SqlxDao query method `{}.{}` only supports required positional parameters in v1",
                    dao_name, method.method.name
                ))
                .with_label(SourceLabel::new(
                    param.span.file_id,
                    param.span.range,
                    "unsupported DAO parameter",
                )),
            );
        }
    }
}

fn is_supported_dao_ok_type(ty: &TypeIr, row_names: &HashSet<&str>) -> bool {
    if ty.is_named(DART_EXEC_RESULT) || ty.is_named(DART_UNIT) || is_supported_scalar_type(ty) {
        return true;
    }
    if ty.is_named(DART_LIST) {
        let Some(item) = ty.args().first() else {
            return false;
        };
        return item.is_named(DART_ROW) || item.name().is_some_and(|name| row_names.contains(name));
    }
    ty.name().is_some_and(|name| row_names.contains(name))
}
