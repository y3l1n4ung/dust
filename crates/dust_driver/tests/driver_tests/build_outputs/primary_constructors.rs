use std::fs;

use dust_driver::{BuildRequest, run_build};

use crate::support::{generated_output, make_workspace, write_file};

#[test]
fn build_writes_derive_output_for_primary_constructor_class() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/primary_user.dart"),
        r#"
part 'primary_user.g.dart';

@Derive([ToString(), Eq(), CopyWith()])
class PrimaryUser(var String id, var int? age);
"#,
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });
    let output = fs::read_to_string(workspace.path().join("lib/primary_user.g.dart")).unwrap();

    assert_eq!(result.diagnostics, vec![]);
    assert_eq!(
        output,
        generated_output(
            r#"part of 'primary_user.dart';

const Object _undefined = Object();

mixin _$PrimaryUser {
  @override
  String toString() {
    final self = this as PrimaryUser;
    return 'PrimaryUser('
        'id: ${self.id}, '
        'age: ${self.age}'
        ')';
  }

  @override
  bool operator ==(Object other) {
    final self = this as PrimaryUser;
    return identical(this, other) ||
        other is PrimaryUser &&
            runtimeType == other.runtimeType &&
            other.id == self.id &&
            other.age == self.age;
  }

  @override
  int get hashCode {
    final self = this as PrimaryUser;
    return Object.hashAll([
      runtimeType,
      self.id,
      self.age,
    ]);
  }

  PrimaryUser copyWith({
    String? id,
    Object? age = _undefined,
  }) {
    final self = this as PrimaryUser;
    return PrimaryUser(
      id ?? self.id,
      identical(age, _undefined)
          ? self.age
          : age as int?,
    );
  }
}
"#
        )
    );
}

#[test]
fn build_writes_serde_output_for_primary_constructor_class() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/primary_profile.dart"),
        r#"
part 'primary_profile.g.dart';

@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)
class PrimaryProfile({required var String id, required var String displayName});
"#,
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });
    let output = fs::read_to_string(workspace.path().join("lib/primary_profile.g.dart")).unwrap();

    assert_eq!(result.diagnostics, vec![]);
    assert_eq!(
        output,
        generated_output(
            r#"part of 'primary_profile.dart';

mixin _$PrimaryProfile {
  Map<String, Object?> toJson() => _$PrimaryProfileToJson(this as PrimaryProfile);
}

Map<String, Object?> _$PrimaryProfileToJson(PrimaryProfile instance) {
  return <String, Object?>{
    'id': instance.id,
    'display_name': instance.displayName,
  };
}
// factory PrimaryProfile.fromJson(Map<String, Object?> json) => _$PrimaryProfileFromJson(json);
PrimaryProfile _$PrimaryProfileFromJson(Map<String, Object?> json) {
  const allowedKeys = <String>{'id', 'display_name'};
  for (final key in json.keys) {
    if (!allowedKeys.contains(key)) {
      throw ArgumentError.value(key, 'json', 'unknown key for PrimaryProfile');
    }
  }

  final idValue = JsonHelper.as<String>(json['id'], 'id', 'String');
  final displayNameValue = JsonHelper.as<String>(
    json['display_name'],
    'display_name',
    'String',
  );

  return PrimaryProfile(id: idValue, displayName: displayNameValue);
}
"#
        )
    );
}
