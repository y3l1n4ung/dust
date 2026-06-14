use dust_dart_emit::{
    DART_BOOL, DART_DATE_TIME, DART_DOUBLE, DART_EXEC_RESULT, DART_FUTURE, DART_INT, DART_LIST,
    DART_NUM, DART_RESULT, DART_ROW, DART_STRING, DART_UNIT,
};
use dust_ir::{ConfigApplicationIr, TypeIr};

use crate::plugin::model::{FetchMode, QueryFunction};

use dust_dart_syntax::parse_static_dart_string_literal;

pub(super) fn parse_query_config(config: &ConfigApplicationIr) -> (String, bool) {
    if let Some(sql) = config
        .positional_argument_source(0)
        .and_then(parse_static_dart_string_literal)
    {
        return (sql, true);
    }
    if let Some(value) = config.named_expression_source("sql") {
        return parse_static_dart_string_literal(&value)
            .map(|sql| (sql, true))
            .unwrap_or_else(|| (String::new(), false));
    }
    (String::new(), false)
}

pub(super) fn query_shape_from_return(
    ok_type: Option<&TypeIr>,
) -> (QueryFunction, FetchMode, Option<String>, Option<TypeIr>) {
    let Some(ok_type) = ok_type else {
        return (QueryFunction::Raw, FetchMode::Raw, None, None);
    };
    if ok_type.is_named(DART_EXEC_RESULT) || ok_type.is_named(DART_UNIT) {
        return (QueryFunction::Execute, FetchMode::Execute, None, None);
    }
    if ok_type.is_named(DART_LIST) {
        let Some(item) = ok_type.args().first() else {
            return (QueryFunction::Raw, FetchMode::Raw, None, None);
        };
        if item.is_named(DART_ROW) {
            return (QueryFunction::Raw, FetchMode::Raw, None, None);
        }
        return (
            QueryFunction::As,
            FetchMode::All,
            item.name().map(str::to_owned),
            None,
        );
    }
    if is_scalar_type(ok_type) {
        return (
            QueryFunction::Scalar,
            if ok_type.is_nullable() {
                FetchMode::Optional
            } else {
                FetchMode::One
            },
            None,
            Some(ok_type.clone()),
        );
    }
    (
        QueryFunction::As,
        if ok_type.is_nullable() {
            FetchMode::Optional
        } else {
            FetchMode::One
        },
        ok_type.name().map(str::to_owned),
        None,
    )
}

pub(crate) fn result_ok_type(return_type: &TypeIr) -> Option<&TypeIr> {
    let future = return_type
        .is_named(DART_FUTURE)
        .then(|| return_type.args().first())
        .flatten()?;
    let result = future.is_named(DART_RESULT).then_some(future)?;
    result.args().first()
}

fn is_scalar_type(ty: &TypeIr) -> bool {
    matches!(
        ty.name(),
        Some(DART_STRING | DART_INT | DART_DOUBLE | DART_NUM | DART_BOOL | DART_DATE_TIME)
    )
}

pub(super) fn parse_fetch_method(function: QueryFunction, method: Option<&str>) -> FetchMode {
    match method {
        Some("fetchOptional") => return FetchMode::Optional,
        Some("fetchOne") => return FetchMode::One,
        Some("fetchAll") => return FetchMode::All,
        Some("fetch") => return FetchMode::Raw,
        Some("execute") => return FetchMode::Execute,
        _ => {}
    }
    match function {
        QueryFunction::Execute => FetchMode::Execute,
        QueryFunction::Raw => FetchMode::Raw,
        _ => FetchMode::One,
    }
}
