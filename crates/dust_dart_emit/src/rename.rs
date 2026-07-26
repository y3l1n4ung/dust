use dust_ir::{SerdeRenameRuleIr, apply_serde_rename_rule};

/// Applies a serde rename rule without exposing the helper publicly.
pub(crate) fn apply_rename_rule_impl(source: &str, rule: SerdeRenameRuleIr) -> String {
    apply_serde_rename_rule(source, rule)
}
