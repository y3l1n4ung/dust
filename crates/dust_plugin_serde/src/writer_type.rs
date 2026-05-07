pub(crate) use dust_dart_emit::non_nullable;

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
    dust_dart_emit::apply_rename_rule(source, rule)
}

fn is_simple_receiver(source: &str) -> bool {
    !source.is_empty()
        && source
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.'))
}
