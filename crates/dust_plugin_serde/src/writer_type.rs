pub(crate) use dust_dart_emit::non_nullable;
use heck::{AsKebabCase, AsLowerCamelCase, AsPascalCase, AsSnakeCase, AsTrainCase};

use dust_ir::SerdeRenameRuleIr;

pub(crate) fn non_null_encode_expr(expr: &str) -> String {
    format!("({expr}!)")
}

pub(crate) fn access_receiver(source: &str) -> String {
    if is_simple_receiver(source) {
        source.to_owned()
    } else {
        format!("({source})")
    }
}

pub(crate) fn apply_rename_rule(source: &str, rule: SerdeRenameRuleIr) -> String {
    match rule {
        SerdeRenameRuleIr::LowerCase => source.to_lowercase(),
        SerdeRenameRuleIr::UpperCase => source.to_uppercase(),
        SerdeRenameRuleIr::PascalCase => AsPascalCase(source).to_string(),
        SerdeRenameRuleIr::CamelCase => AsLowerCamelCase(source).to_string(),
        SerdeRenameRuleIr::SnakeCase => AsSnakeCase(source).to_string(),
        SerdeRenameRuleIr::ScreamingSnakeCase => AsSnakeCase(source).to_string().to_uppercase(),
        SerdeRenameRuleIr::KebabCase => AsKebabCase(source).to_string(),
        SerdeRenameRuleIr::ScreamingKebabCase => AsTrainCase(source).to_string().to_uppercase(),
    }
}

fn is_simple_receiver(source: &str) -> bool {
    !source.is_empty()
        && source
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.'))
}
