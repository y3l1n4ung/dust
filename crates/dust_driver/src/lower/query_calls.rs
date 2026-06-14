use dust_diagnostics::Diagnostic;
use dust_ir::{QueryCallIr, QueryFunctionIr, SpanIr};
use dust_parser_dart::ParsedQueryFunction;
use dust_resolver::ResolvedLibrary;

use super::type_parse::lower_type;

pub(super) fn lower_query_calls(
    library: &ResolvedLibrary,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<QueryCallIr> {
    library
        .query_calls
        .iter()
        .map(|query| {
            let type_outcome = lower_type(None, query.type_arg_source.as_deref());
            diagnostics.extend(type_outcome.diagnostics);
            QueryCallIr {
                function: match query.function {
                    ParsedQueryFunction::As => QueryFunctionIr::As,
                    ParsedQueryFunction::Scalar => QueryFunctionIr::Scalar,
                    ParsedQueryFunction::Raw => QueryFunctionIr::Raw,
                    ParsedQueryFunction::Execute => QueryFunctionIr::Execute,
                },
                type_arg: query.type_arg_source.as_ref().map(|_| type_outcome.value),
                type_arg_source: query.type_arg_source.clone(),
                sql: query.sql.clone(),
                sql_source_static: query.sql_source_static,
                parameter_count: query.parameter_count,
                params_source_is_list: query.params_source_is_list,
                fetch_method: query.fetch_method.clone(),
                span: SpanIr::new(library.span.file_id, query.span),
            }
        })
        .collect()
}
