#![cfg(test)]

use super::type_parse::{parse_type, split_top_level_args};
use dust_ir::BuiltinType;

#[test]
fn parses_builtin_and_nullable_types() {
    let ty = parse_type("String?");
    assert!(ty.is_builtin(BuiltinType::String));
    assert!(ty.is_nullable());
}

#[test]
fn parses_nested_generic_types() {
    let ty = parse_type("Map<String, List<int?>>");
    assert!(ty.is_named("Map"));
    assert_eq!(ty.args().len(), 2);
    assert!(ty.args()[1].is_named("List"));
    assert!(ty.args()[1].args()[0].is_builtin(BuiltinType::Int));
    assert!(ty.args()[1].args()[0].is_nullable());
}

#[test]
fn keeps_function_like_types_as_named_fallbacks() {
    let ty = parse_type("void Function(String, int)?");
    assert!(ty.is_function());
    assert!(ty.is_nullable());
}

#[test]
fn parses_record_types_without_falling_back_to_named() {
    let ty = parse_type("({String name, int age})?");
    assert!(ty.is_record());
    assert!(ty.is_nullable());
}

#[test]
fn splits_top_level_args_without_breaking_nested_generics() {
    assert_eq!(
        split_top_level_args("String, Map<String, List<int>>, ({String name, int age})"),
        vec![
            "String",
            "Map<String, List<int>>",
            "({String name, int age})"
        ]
    );
}
