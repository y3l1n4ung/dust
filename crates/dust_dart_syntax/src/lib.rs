#![deny(missing_docs)]
#![doc = "Shared Dart source parsing helpers for Dust parser, IR, and plugins."]

/// Literal-value parsing helpers.
mod literals;
/// Top-level source scanning helpers.
mod scanner;
/// Annotation value parsing helpers.
mod values;

pub use literals::{parse_bool_literal, parse_static_dart_string_literal, parse_string_literal};
pub use scanner::{
    balanced_parenthesized, find_top_level_char, has_top_level_char, normalized_args,
    parse_named_arguments, split_top_level_items, split_top_level_once,
};
pub use values::{
    parse_constructor_list, parse_constructor_name, parse_member_ref, parse_string_list,
    parse_string_map, parse_type_list, parse_type_name,
};
