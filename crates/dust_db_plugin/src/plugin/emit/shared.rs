use dust_dart_emit::{DART_BOOL, DART_DATE_TIME, DART_DOUBLE, DART_INT, DART_NUM, DART_STRING};
use dust_ir::TypeIr;

pub(super) fn lower_first(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!(
        "{}{}",
        first.to_ascii_lowercase(),
        chars.collect::<String>()
    )
}

pub(super) fn escape_dart_string(source: &str) -> String {
    source
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('$', "\\$")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

pub(super) fn render_sql_literal(source: &str) -> String {
    if !source.contains("'''") {
        return format!("r'''{source}'''");
    }
    format!("'{}'", escape_dart_string(source))
}

pub(super) fn is_scalar_type(ty: &TypeIr) -> bool {
    matches!(
        ty.name(),
        Some(DART_STRING | DART_INT | DART_DOUBLE | DART_NUM | DART_BOOL | DART_DATE_TIME)
    )
}
