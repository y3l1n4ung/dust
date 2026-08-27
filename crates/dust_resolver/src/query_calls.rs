use dust_diagnostics::Diagnostic;
use dust_ir::{QueryCallIr, QueryFunctionIr, SpanIr};
use dust_parser_dart::{ParsedQueryCallSurface, ParsedQueryFunction};
use dust_text::FileId;

use crate::lower_type_ir;

/// Normalizes parsed DB query helper calls into canonical query call IR.
pub(crate) fn normalize_query_calls(
    file_id: FileId,
    query_calls: &[ParsedQueryCallSurface],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<QueryCallIr> {
    query_calls
        .iter()
        .map(|query| {
            let type_outcome = lower_type_ir(None, query.type_arg_source.as_deref());
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
                has_row_mapper_argument: query.has_row_mapper_argument,
                span: SpanIr::new(file_id, query.span),
            }
        })
        .collect()
}
