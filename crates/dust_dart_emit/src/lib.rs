#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Shared Dart rendering helpers reused by Dust plugins."]

/// Dart runtime type and symbol names.
mod dart_names;
/// Dart literal escaping.
mod literals;
/// Serde rename rules.
mod rename;
/// Dart source parsing re-exports.
mod source;
/// Template rendering helpers.
mod templates;
/// Dart type rendering.
mod type_render;

pub use dart_names::{
    DART_BIG_INT, DART_BOOL, DART_DATE_TIME, DART_DOUBLE, DART_DYNAMIC, DART_EXEC_RESULT,
    DART_FUTURE, DART_INT, DART_ITERABLE, DART_LIST, DART_MAP, DART_NUM, DART_OBJECT,
    DART_OBJECT_NULLABLE, DART_RESPONSE, DART_RESPONSE_BODY, DART_RESULT, DART_ROW, DART_SET,
    DART_STREAM, DART_STRING, DART_UNIT, DART_URI, DART_VOID,
};

pub use literals::{dart_string_literal, escape_dart_string};

pub use source::{
    balanced_parenthesized, normalized_args, parse_bool_literal, parse_named_arguments,
    parse_static_dart_string_literal, parse_string_literal, split_top_level_items,
    split_top_level_once,
};

pub use templates::render_template;

pub use type_render::{
    DYNAMIC_TYPES, DartTypeRenderer, OBJECT_NULLABLE_TYPES, UnknownTypeRendering, non_nullable,
};

use dust_ir::SerdeRenameRuleIr;

/// Applies Dust's normalized serde rename rule to one Dart identifier.
pub fn apply_rename_rule(source: &str, rule: SerdeRenameRuleIr) -> String {
    rename::apply_rename_rule_impl(source, rule)
}

#[cfg(test)]
mod tests {
    use dust_ir::SerdeRenameRuleIr;

    use super::{apply_rename_rule, render_template};

    #[derive(serde::Serialize)]
    struct TemplateContext<'a> {
        name: &'a str,
    }

    #[test]
    fn public_facade_applies_every_rename_rule() {
        assert_eq!(
            apply_rename_rule("displayName", SerdeRenameRuleIr::LowerCase),
            "displayname"
        );
        assert_eq!(
            apply_rename_rule("displayName", SerdeRenameRuleIr::UpperCase),
            "DISPLAYNAME"
        );
        assert_eq!(
            apply_rename_rule("display_name", SerdeRenameRuleIr::PascalCase),
            "DisplayName"
        );
        assert_eq!(
            apply_rename_rule("display_name", SerdeRenameRuleIr::CamelCase),
            "displayName"
        );
        assert_eq!(
            apply_rename_rule("displayName", SerdeRenameRuleIr::SnakeCase),
            "display_name"
        );
        assert_eq!(
            apply_rename_rule("displayName", SerdeRenameRuleIr::ScreamingSnakeCase),
            "DISPLAY_NAME"
        );
        assert_eq!(
            apply_rename_rule("displayName", SerdeRenameRuleIr::KebabCase),
            "display-name"
        );
        assert_eq!(
            apply_rename_rule("displayName", SerdeRenameRuleIr::ScreamingKebabCase),
            "DISPLAY-NAME"
        );
    }

    #[test]
    fn public_facade_renders_templates_without_trailing_newline() {
        let rendered = render_template(
            "greeting",
            "hello {{ name }}\n",
            TemplateContext { name: "Dust" },
        );

        assert_eq!(rendered, "hello Dust");
    }
}
