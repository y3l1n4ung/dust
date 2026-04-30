use std::fs;

use dust_diagnostics::Diagnostic;
use dust_emitter::write_library;
use dust_ir::{ClassIr, ClassKindIr, LibraryIr, ParamKind, TypeIr};
use dust_plugin_api::{PluginContribution, PluginRegistry};
use dust_plugin_derive::register_plugin;
use tempfile::tempdir;

use super::support::{
    FakePlugin, constructor, constructor_param, field, sample_library, span, trait_app,
};

#[test]
fn write_library_writes_real_file_and_skips_rewrite_when_unchanged() {
    let temp = tempdir().unwrap();
    let output_path = temp.path().join("lib/user.g.dart");
    let library = sample_library(output_path.display().to_string());

    let mut contribution = PluginContribution::default();
    contribution.push_mixin_member("User", "User copyWith({String? id}) => User();");

    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "fake",
            requested: Vec::new(),
            diagnostics: Vec::new(),
            contribution,
        }))
        .unwrap();

    let first = write_library(&library, &registry).unwrap();
    let second = write_library(&library, &registry).unwrap();
    let written = fs::read_to_string(&output_path).unwrap();

    assert!(first.written);
    assert!(first.changed);
    assert!(!second.written);
    assert!(!second.changed);
    assert_eq!(written, first.source);
}

#[test]
fn write_library_skips_output_when_validation_has_errors() {
    let temp = tempdir().unwrap();
    let output_path = temp.path().join("lib/user.g.dart");
    let library = sample_library(output_path.display().to_string());

    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "broken",
            requested: Vec::new(),
            diagnostics: vec![Diagnostic::error("broken derive configuration")],
            contribution: PluginContribution::default(),
        }))
        .unwrap();

    let result = write_library(&library, &registry).unwrap();

    assert!(!result.written);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(!output_path.exists());
}

#[test]
fn emitter_generates_real_multi_class_output_with_derive_plugin() {
    let temp = tempdir().unwrap();
    let output_path = temp.path().join("lib/models.g.dart");
    let library = LibraryIr {
        source_path: "lib/models.dart".to_owned(),
        output_path: output_path.display().to_string(),
        span: span(0, 200),
        classes: vec![
            ClassIr {
                kind: ClassKindIr::Class,
                name: "User".to_owned(),
                is_abstract: false,
                superclass_name: None,
                span: span(0, 80),
                fields: vec![
                    field("id", TypeIr::string()),
                    field("age", TypeIr::int().nullable()),
                ],
                constructors: vec![constructor(
                    None,
                    vec![
                        constructor_param("id", TypeIr::string(), ParamKind::Positional),
                        constructor_param("age", TypeIr::int().nullable(), ParamKind::Positional),
                    ],
                )],
                traits: vec![
                    trait_app("derive_annotation::ToString"),
                    trait_app("derive_annotation::Eq"),
                    trait_app("derive_annotation::CopyWith"),
                ],
                serde: None,
            },
            ClassIr {
                kind: ClassKindIr::Class,
                name: "Team".to_owned(),
                is_abstract: false,
                superclass_name: None,
                span: span(81, 160),
                fields: vec![field("name", TypeIr::string())],
                constructors: vec![constructor(
                    None,
                    vec![constructor_param(
                        "name",
                        TypeIr::string(),
                        ParamKind::Positional,
                    )],
                )],
                traits: vec![trait_app("derive_annotation::CopyWith")],
                serde: None,
            },
        ],
        enums: Vec::new(),
    };

    let mut registry = PluginRegistry::new();
    registry.register(Box::new(register_plugin())).unwrap();

    let result = write_library(&library, &registry).unwrap();
    let written = fs::read_to_string(&output_path).unwrap();

    assert!(result.written);
    assert!(written.contains("// coverage:ignore-file"));
    assert!(written.contains("// ignore_for_file: type=lint"));
    assert!(written.contains("part of 'models.dart';"));
    assert!(written.contains("const Object _undefined = Object();"));
    assert!(written.contains("mixin _$UserDust {"));
    assert!(written.contains("User get _dustSelf => this as User;"));
    assert!(written.contains("String toString() {\n    return 'User('"));
    assert!(written.contains("'id: ${_dustSelf.id}, '"));
    assert!(written.contains("'age: ${_dustSelf.age}'"));
    assert!(written.contains(
        "int get hashCode => Object.hashAll([\n    runtimeType,\n    _dustSelf.id,\n    _dustSelf.age,\n  ]);"
    ));
    assert!(written.contains("User copyWith({"));
    assert!(!written.contains("final nextIdSource = id ?? _dustSelf.id;"));
    assert!(written.contains("id ?? _dustSelf.id,"));
    assert!(written.contains("return User("));
    assert!(written.contains("mixin _$TeamDust {"));
    assert!(written.contains("Team copyWith({"));
    assert!(!written.contains("final clonedName = _dustSelf.name;"));
    assert!(written.contains("_dustSelf.name,"));
    assert!(written.contains("return Team("));
    assert!(!written.contains("return   "));
}
