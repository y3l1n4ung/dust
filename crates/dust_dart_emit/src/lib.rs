#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Shared Dart rendering helpers reused by Dust plugins."]

mod rename;
mod source;
mod templates;
mod type_render;

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
