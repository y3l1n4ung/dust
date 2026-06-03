use dust_dart_emit::{DART_LIST, DART_MAP, DART_RESPONSE_BODY, DART_STRING};
use dust_diagnostics::SourceLabel;
use dust_ir::{BuiltinType, SpanIr, TypeIr};
use dust_plugin_api::short_symbol_name;

pub(super) fn config_name(symbol: &str) -> &str {
    short_symbol_name(symbol)
}

pub(super) fn escape_single_quoted(source: &str) -> String {
    source
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('$', "\\$")
}

pub(super) fn label(span: SpanIr, message: impl Into<String>) -> SourceLabel {
    SourceLabel::new(span.file_id, span.range, message)
}

pub(super) fn extract_path_placeholders(path: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut cursor = 0_usize;
    while let Some(start) = path[cursor..].find('{') {
        let start = cursor + start;
        let Some(end) = path[start + 1..].find('}') else {
            break;
        };
        let end = start + 1 + end;
        placeholders.push(path[start + 1..end].to_owned());
        cursor = end + 1;
    }
    placeholders
}

pub(super) fn type_name_is(ty: &TypeIr, expected: &str) -> bool {
    ty.name()
        .map(|name| name == expected || name.rsplit('.').next() == Some(expected))
        .unwrap_or(false)
}

pub(super) fn is_string_keyed_map(ty: &TypeIr) -> bool {
    type_name_is(ty, DART_MAP) && ty.args().len() == 2 && ty.args()[0].is_named(DART_STRING)
}

pub(super) fn is_response_body_type(ty: &TypeIr) -> bool {
    type_name_is(ty, DART_RESPONSE_BODY)
}

pub(super) fn is_list_of_int_type(ty: &TypeIr) -> bool {
    type_name_is(ty, DART_LIST) && ty.args().len() == 1 && ty.args()[0].is_builtin(BuiltinType::Int)
}

pub(super) fn is_string_type(ty: &TypeIr) -> bool {
    ty.is_named(DART_STRING)
}

pub(super) fn has_import(imports: &[String], uri: &str) -> bool {
    imports.iter().any(|import| import == uri)
}
