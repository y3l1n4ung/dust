#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Shared Dart rendering helpers reused by Dust plugins."]

mod rename;
mod type_render;

pub use type_render::{
    DYNAMIC_TYPES, DartTypeRenderer, OBJECT_NULLABLE_TYPES, UnknownTypeRendering, non_nullable,
};

use dust_ir::SerdeRenameRuleIr;

/// Applies Dust's normalized serde rename rule to one Dart identifier.
pub fn apply_rename_rule(source: &str, rule: SerdeRenameRuleIr) -> String {
    rename::apply_rename_rule_impl(source, rule)
}
