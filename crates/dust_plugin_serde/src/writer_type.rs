pub(crate) use dust_dart_emit::non_nullable;

use dust_ir::SerdeRenameRuleIr;

/// Renders an expression with a non-null assertion for generated encoding.
pub(crate) fn non_null_encode_expr(expr: &str) -> String {
    format!("({expr}!)")
}

/// Returns a safe receiver expression for calling codec methods.
pub(crate) fn access_receiver(source: &str) -> String {
    if is_simple_receiver(source) {
        source.to_owned()
    } else {
        format!("({source})")
    }
}

/// Applies a serde rename rule to a source identifier.
pub(crate) fn apply_rename_rule(source: &str, rule: SerdeRenameRuleIr) -> String {
    dust_dart_emit::apply_rename_rule(source, rule)
}

/// Returns true when a receiver expression needs no parentheses.
fn is_simple_receiver(source: &str) -> bool {
    !source.is_empty()
        && source
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.'))
}
