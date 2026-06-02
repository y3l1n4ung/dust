use dust_ir::{QueryCallIr, QueryFunctionIr, SpanIr, TypeIr};
use dust_text::{FileId, TextRange};

pub(crate) fn query_as(
    row_type: &str,
    sql: &str,
    parameter_count: usize,
    fetch_method: &str,
    start: u32,
) -> QueryCallIr {
    query_call(
        QueryFunctionIr::As,
        Some((row_type.to_owned(), TypeIr::named(row_type))),
        sql,
        parameter_count,
        fetch_method,
        start,
    )
}

pub(crate) fn query_scalar(
    ty: TypeIr,
    sql: &str,
    parameter_count: usize,
    fetch_method: &str,
    start: u32,
) -> QueryCallIr {
    let source = ty.name().unwrap_or("dynamic").to_owned();
    query_call(
        QueryFunctionIr::Scalar,
        Some((source, ty)),
        sql,
        parameter_count,
        fetch_method,
        start,
    )
}

pub(crate) fn query_raw(sql: &str, parameter_count: usize, start: u32) -> QueryCallIr {
    query_call(
        QueryFunctionIr::Raw,
        None,
        sql,
        parameter_count,
        "fetch",
        start,
    )
}

pub(crate) fn query_execute(sql: &str, parameter_count: usize, start: u32) -> QueryCallIr {
    query_call(
        QueryFunctionIr::Execute,
        None,
        sql,
        parameter_count,
        "execute",
        start,
    )
}

fn query_call(
    function: QueryFunctionIr,
    type_arg: Option<(String, TypeIr)>,
    sql: &str,
    parameter_count: usize,
    fetch_method: &str,
    start: u32,
) -> QueryCallIr {
    QueryCallIr {
        function,
        type_arg: type_arg.as_ref().map(|(_, ty)| ty.clone()),
        type_arg_source: type_arg.map(|(source, _)| source),
        sql: sql.to_owned(),
        sql_source_static: true,
        parameter_count,
        params_source_is_list: true,
        fetch_method: Some(fetch_method.to_owned()),
        span: SpanIr::new(FileId::new(7), TextRange::new(start, start + 1)),
    }
}
