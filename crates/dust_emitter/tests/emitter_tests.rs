use std::fs;

use dust_diagnostics::Diagnostic;
use dust_emitter::{emit_library, write_library};
use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr, ParamKind, SpanIr,
    SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::{DustPlugin, PluginContribution, PluginRegistry, SymbolPlan};
use dust_plugin_derive::register_plugin;
use dust_text::{FileId, TextRange};
use tempfile::tempdir;

fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(start, end))
}

fn field(name: &str, ty: TypeIr) -> FieldIr {
    FieldIr {
        name: name.to_owned(),
        ty,
        span: span(10, 20),
        has_default: false,
        serde: None,
    }
}

fn constructor_param(name: &str, ty: TypeIr, kind: ParamKind) -> ConstructorParamIr {
    ConstructorParamIr {
        name: name.to_owned(),
        ty,
        span: span(20, 30),
        kind,
        has_default: false,
    }
}

fn constructor(name: Option<&str>, params: Vec<ConstructorParamIr>) -> ConstructorIr {
    ConstructorIr {
        name: name.map(str::to_owned),
        span: span(30, 40),
        params,
    }
}

fn class(name: &str, fields: Vec<FieldIr>, constructors: Vec<ConstructorIr>) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        superclass_name: None,
        span: span(0, 100),
        fields,
        constructors,
        traits: Vec::new(),
        serde: None,
    }
}

fn trait_app(symbol: &str) -> TraitApplicationIr {
    TraitApplicationIr {
        symbol: SymbolId::new(symbol),
        span: span(1, 5),
    }
}

fn sample_library(output_path: String) -> LibraryIr {
    LibraryIr {
        source_path: "lib/user.dart".to_owned(),
        output_path,
        span: span(0, 120),
        classes: vec![class(
            "User",
            Vec::new(),
            vec![constructor(None, Vec::new())],
        )],
    }
}

struct FakePlugin {
    name: &'static str,
    requested: Vec<&'static str>,
    diagnostics: Vec<Diagnostic>,
    contribution: PluginContribution,
}

impl DustPlugin for FakePlugin {
    fn plugin_name(&self) -> &'static str {
        self.name
    }

    fn validate(&self, _library: &LibraryIr) -> Vec<Diagnostic> {
        self.diagnostics.clone()
    }

    fn requested_symbols(&self, _library: &LibraryIr) -> Vec<String> {
        self.requested
            .iter()
            .map(|name| (*name).to_owned())
            .collect()
    }

    fn emit(&self, _library: &LibraryIr, _plan: &SymbolPlan) -> PluginContribution {
        self.contribution.clone()
    }
}

#[test]
fn emitter_writes_sections_in_fixed_order_and_wraps_mixins() {
    let mut contribution = PluginContribution::default();
    contribution.push_mixin_member("User", "@override\nString toString() => 'User()';");
    contribution
        .support_types
        .push("typedef UserMap = Map<String, Object?>;".to_owned());
    contribution
        .top_level_functions
        .push("User _$UserFromJson(Map<String, Object?> json) => User();".to_owned());

    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "fake",
            requested: vec!["_undefined"],
            diagnostics: Vec::new(),
            contribution,
        }))
        .unwrap();

    let result = emit_library(
        &sample_library("lib/user.g.dart".to_owned()),
        &registry,
        None,
    );
    let source = result.source;

    let header = source.find("// GENERATED CODE").unwrap();
    let part_of = source.find("part of 'user.dart';").unwrap();
    let helper = source.find("const Object _undefined = Object();").unwrap();
    let mixin = source.find("mixin _$UserDust {").unwrap();
    let support = source.find("typedef UserMap").unwrap();
    let function = source.find("_$UserFromJson").unwrap();

    assert!(
        header < part_of
            && part_of < helper
            && helper < mixin
            && mixin < support
            && support < function
    );
    assert!(source.contains("// coverage:ignore-file"));
    assert!(source.contains("// ignore_for_file: type=lint"));
    assert!(source.contains(
        "// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark"
    ));
    assert!(source.contains("mixin _$UserDust {"));
    assert!(source.contains("  User get _dustSelf => this as User;"));
    assert!(source.contains("  @override\n  String toString() => 'User()';"));
}

#[test]
fn emitter_dedupes_helpers_and_symbol_reservations() {
    let mut registry = PluginRegistry::new();

    for name in ["a", "b"] {
        let mut contribution = PluginContribution::default();
        contribution
            .shared_helpers
            .push("typedef UserMap = Map<String, Object?>;".to_owned());
        registry
            .register(Box::new(FakePlugin {
                name,
                requested: vec!["_undefined"],
                diagnostics: Vec::new(),
                contribution,
            }))
            .unwrap();
    }

    let result = emit_library(
        &sample_library("lib/user.g.dart".to_owned()),
        &registry,
        None,
    );
    assert_eq!(result.symbols.reserved().len(), 1);
    assert_eq!(
        result
            .source
            .matches("const Object _undefined = Object();")
            .count(),
        1
    );
    assert_eq!(
        result
            .source
            .matches("typedef UserMap = Map<String, Object?>;")
            .count(),
        1
    );
}

#[test]
fn emitter_sets_changed_false_when_output_matches_previous() {
    let registry = PluginRegistry::new();
    let library = sample_library("lib/user.g.dart".to_owned());

    let first = emit_library(&library, &registry, None);
    let second = emit_library(&library, &registry, Some(&first.source));

    assert!(first.changed);
    assert!(!second.changed);
}

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
