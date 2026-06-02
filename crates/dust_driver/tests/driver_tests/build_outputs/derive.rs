use std::fs;

use dust_driver::{BuildRequest, run_build};

use crate::support::{generated_output, make_workspace, write_file};

#[test]
fn build_writes_real_outputs_for_multiple_libraries_and_classes() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/models.dart"),
        "part 'models.g.dart';\n\
         @Derive([ToString(), Eq(), CopyWith()])\n\
         class User {\n\
           final String id;\n\
           final int? age;\n\
           const User(this.id, this.age);\n\
         }\n\
         @CopyWith()\n\
         class Team {\n\
           final String name;\n\
           const Team(this.name);\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/request.dart"),
        "part 'request.g.dart';\n\
         @CopyWith()\n\
         class Request {\n\
           final String path;\n\
           final Map<String, String> headers;\n\
           const Request.create({required this.path, required this.headers});\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    let models_output = fs::read_to_string(workspace.path().join("lib/models.g.dart")).unwrap();
    let request_output = fs::read_to_string(workspace.path().join("lib/request.g.dart")).unwrap();

    assert!(!result.has_errors());
    assert_eq!(result.build_artifacts.len(), 2);
    assert_eq!(result.cache.as_ref().unwrap().misses, 2);
    assert_eq!(result.cache.as_ref().unwrap().hits, 0);
    assert!(
        result
            .build_artifacts
            .iter()
            .all(|artifact| artifact.written)
    );
    assert_eq!(
        models_output,
        generated_output(
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
"#
        )
    );
    assert_eq!(
        request_output,
        generated_output(
            r#"part of 'request.dart';

const DeepCollectionEquality _deepCollectionEquality = DeepCollectionEquality();

mixin _$Request {
  Request copyWith({
    String? path,
    Map<String, String>? headers,
  }) {
    final self = this as Request;
    final nextHeaders = Map<String, String>.of(headers ?? self.headers);

    return Request.create(
      path: path ?? self.path,
      headers: nextHeaders,
    );
  }
}
"#
        )
    );
}
