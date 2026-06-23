use dust_ir::SerdeRenameRuleIr;
use heck::{AsKebabCase, AsLowerCamelCase, AsPascalCase, AsSnakeCase, AsTrainCase};

/// Applies a serde rename rule without exposing the helper publicly.
pub(crate) fn apply_rename_rule_impl(source: &str, rule: SerdeRenameRuleIr) -> String {
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
