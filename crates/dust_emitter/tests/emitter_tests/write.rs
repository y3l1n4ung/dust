use std::fs;

use dust_diagnostics::Diagnostic;
use dust_emitter::write_library;
use dust_ir::{ClassIr, ClassKindIr, LibraryIr, ParamKind, TypeIr};
use dust_plugin_api::{PluginContribution, PluginRegistry};
use dust_plugin_derive::register_plugin;
use tempfile::tempdir;

use super::support::{
    FakePlugin, constructor, constructor_param, field, generated_output, sample_library, span,
    trait_app,
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
        package_root: ".".to_owned(),
        package_name: "dust_test".to_owned(),
        source_path: "lib/models.dart".to_owned(),
        output_path: output_path.display().to_string(),
        imports: Vec::new(),
        library: None,
        library_annotations: Vec::new(),
        import_directives: Vec::new(),
        export_directives: Vec::new(),
        part_directives: Vec::new(),
        part_of: None,
        span: span(0, 200),
        classes: vec![
            ClassIr {
                kind: ClassKindIr::Class,
                name: "User".to_owned(),
                is_abstract: false,
                is_interface: false,
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
                methods: Vec::new(),
                traits: vec![
                    trait_app("dust_dart::ToString"),
                    trait_app("dust_dart::Eq"),
                    trait_app("dust_dart::CopyWith"),
                ],
                configs: Vec::new(),
                serde: None,
            },
            ClassIr {
                kind: ClassKindIr::Class,
                name: "Team".to_owned(),
                is_abstract: false,
                is_interface: false,
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
                methods: Vec::new(),
                traits: vec![trait_app("dust_dart::CopyWith")],
                configs: Vec::new(),
                serde: None,
            },
        ],
        mixins: Vec::new(),
        extensions: Vec::new(),
        extension_types: Vec::new(),
        functions: Vec::new(),
        variables: Vec::new(),
        typedefs: Vec::new(),
        enums: Vec::new(),
        query_calls: Vec::new(),
    };

    let mut registry = PluginRegistry::new();
    registry.register(Box::new(register_plugin())).unwrap();

    let result = write_library(&library, &registry).unwrap();
    let written = fs::read_to_string(&output_path).unwrap();
    let expected = generated_output(
        r#"part of 'models.dart';

const Object _undefined = Object();

mixin _$User {
  @override
  String toString() {
    final self = this as User;
    return 'User('
        'id: ${self.id}, '
        'age: ${self.age}'
        ')';
  }

  @override
  bool operator ==(Object other) {
    final self = this as User;
    return identical(this, other) ||
        other is User &&
            runtimeType == other.runtimeType &&
            other.id == self.id &&
            other.age == self.age;
  }

  @override
  int get hashCode {
    final self = this as User;
    return Object.hashAll([
      runtimeType,
      self.id,
      self.age,
    ]);
  }

  User copyWith({
    String? id,
    Object? age = _undefined,
  }) {
    final self = this as User;
    return User(
      id ?? self.id,
      identical(age, _undefined)
          ? self.age
          : age as int?,
    );
  }
}

mixin _$Team {
  Team copyWith({
    String? name,
  }) {
    final self = this as Team;
    return Team(
      name ?? self.name,
    );
  }
}
"#,
    );

    assert!(result.written);
    assert_eq!(written, expected);
}
