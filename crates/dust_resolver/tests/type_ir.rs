//! Integration tests for resolver-owned Dart type lowering.

use dust_ir::BuiltinType;
use dust_parser_dart::ParsedTypeSurface;
use dust_resolver::lower_type_ir;
use dust_text::TextRange;

#[test]
fn lowers_builtin_and_nullable_types() {
    let outcome = lower_type_ir(None, Some("String?"));

    assert!(outcome.diagnostics.is_empty());
    assert!(outcome.value.is_builtin(BuiltinType::String));
    assert!(outcome.value.is_nullable());
}

#[test]
fn lowers_nested_generic_types() {
    let outcome = lower_type_ir(None, Some("Map<String, List<int?>>"));

    assert!(outcome.diagnostics.is_empty());
    assert!(outcome.value.is_named("Map"));
    assert_eq!(outcome.value.args().len(), 2);
    assert!(outcome.value.args()[1].is_named("List"));
    assert!(outcome.value.args()[1].args()[0].is_builtin(BuiltinType::Int));
    assert!(outcome.value.args()[1].args()[0].is_nullable());
}

#[test]
fn keeps_function_and_record_types_structured() {
    let function = lower_type_ir(None, Some("void Function(String, int)?"));
    let record = lower_type_ir(None, Some("({String name, int age})?"));

    assert!(function.value.is_function());
    assert!(function.value.is_nullable());
    assert!(record.value.is_record());
    assert!(record.value.is_nullable());
}

#[test]
fn lowers_parser_owned_type_surface_before_raw_source_fallback() {
    let parsed =
        ParsedTypeSurface::parse("Map<String, List<int?>>", TextRange::new(0_u32, 23_u32)).unwrap();

    let outcome = lower_type_ir(Some(&parsed), Some("Object"));

    assert!(outcome.diagnostics.is_empty());
    assert!(outcome.value.is_named("Map"));
    assert_eq!(outcome.value.args().len(), 2);
    assert!(outcome.value.args()[1].is_named("List"));
    assert!(outcome.value.args()[1].args()[0].is_builtin(BuiltinType::Int));
    assert!(outcome.value.args()[1].args()[0].is_nullable());
}
