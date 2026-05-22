use std::fs;

use dust_diagnostics::render_to_string_with_files;
use dust_driver::{BuildRequest, run_build};

use super::support::{generated_output, make_workspace, write_file};

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
      identical(age, _undefined) ? self.age : age as int?,
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

#[test]
fn build_writes_real_serde_outputs() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/profile.dart"),
        "part 'profile.g.dart';\n\
         @Derive([Serialize(), Deserialize()])\n\
         @SerDe(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)\n\
         class Profile {\n\
           const Profile({required this.id, this.displayName, this.tags = const ['guest']});\n\
           final String id;\n\
           @SerDe(rename: 'display_name', aliases: ['displayName'])\n\
           final String? displayName;\n\
           @SerDe(defaultValue: const ['guest'])\n\
           final List<String> tags;\n\
           factory Profile.fromJson(Map<String, Object?> json) => _$ProfileFromJson(json);\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/account.dart"),
        "part 'account.g.dart';\n\
         class Profile {\n\
           const Profile({required this.id});\n\
           final String id;\n\
           factory Profile.fromJson(Map<String, Object?> json) => _$ProfileFromJson(json);\n\
           Map<String, Object?> toJson() => _$ProfileToJson(this);\n\
         }\n\
         @Derive([Serialize(), Deserialize()])\n\
         class Account {\n\
           const Account({required this.profile, required this.metrics, required this.archived});\n\
           final Profile profile;\n\
           final Map<String, List<int>> metrics;\n\
           final bool archived;\n\
           factory Account.fromJson(Map<String, Object?> json) => _$AccountFromJson(json);\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });

    let profile_output = fs::read_to_string(workspace.path().join("lib/profile.g.dart")).unwrap();
    let account_output = fs::read_to_string(workspace.path().join("lib/account.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_eq!(
        profile_output,
        generated_output(
            r#"part of 'profile.dart';

const DeepCollectionEquality _deepCollectionEquality = DeepCollectionEquality();
Never _jsonTypeError(Object? value, String key, String expected) => throw ArgumentError.value(value, key, 'expected $expected');
T _jsonAs<T>(Object? value, String key, String expected) => value is T ? value : _jsonTypeError(value, key, expected);
T _jsonParseString<T>(Object? value, String key, String expected, T? Function(String value) parse) => parse(_jsonAs<String>(value, key, 'String')) ?? _jsonTypeError(value, key, expected);
List<Object?> _jsonAsList(Object? value, String key) => _jsonAs<List>(value, key, 'List<Object?>').cast<Object?>();

Map<String, Object?> _jsonAsMap(Object? value, String key) {
  final map = _jsonAs<Map>(value, key, 'Map<String, Object?>');
  try {
    return Map<String, Object?>.from(map);
  } on TypeError {
    _jsonTypeError(value, key, 'Map<String, Object?>');
  }
}
DateTime _jsonAsDateTime(Object? value, String key) => _jsonParseString(value, key, 'ISO-8601 DateTime string', DateTime.tryParse);
Uri _jsonAsUri(Object? value, String key) => _jsonParseString(value, key, 'Uri string', Uri.tryParse);
BigInt _jsonAsBigInt(Object? value, String key) => _jsonParseString(value, key, 'BigInt string', BigInt.tryParse);
T _jsonDecodeWithCodec<T>(dynamic codec, Object? value, String key) {
  if (value == null) {
    throw ArgumentError.value(value, key, 'expected value for SerDeCodec');
  }
  try {
    return codec.deserialize(value as dynamic) as T;
  } catch (error) {
    throw ArgumentError.value(value, key, 'failed SerDeCodec decode: $error');
  }
}

mixin _$Profile {
  Map<String, Object?> toJson() => _$ProfileToJson(this as Profile);
}

Map<String, Object?> _$ProfileToJson(Profile instance) {
  return <String, Object?>{
    'id': instance.id,
    'display_name': instance.displayName,
    'tags': instance.tags.map((item) => item).toList(),
  };
}
// factory Profile.fromJson(Map<String, Object?> json) => _$ProfileFromJson(json);
Profile _$ProfileFromJson(Map<String, Object?> json) {
  const allowedKeys = <String>{'id', 'display_name', 'displayName', 'tags'};
  for (final key in json.keys) {
    if (!allowedKeys.contains(key)) {
      throw ArgumentError.value(key, 'json', 'unknown key for Profile');
    }
  }

  final idValue = _jsonAs<String>(json['id'], 'id', 'String');
  final rawDisplayNameKey = json.containsKey('display_name') ? 'display_name' : json.containsKey('displayName') ? 'displayName' : 'display_name';
  final rawDisplayName = json.containsKey('display_name') ? json['display_name'] : json.containsKey('displayName') ? json['displayName'] : null;
  final displayNameValue = rawDisplayName == null
                           ? null
                           : _jsonAs<String>(rawDisplayName, rawDisplayNameKey, 'String');
  final tagsValue = json.containsKey('tags') ? _jsonAsList(json['tags'], 'tags').map((item) => _jsonAs<String>(item, 'tags', 'String')).toList() : const ['guest'];

  return Profile(
    id: idValue,
    displayName: displayNameValue,
    tags: tagsValue,
  );
}
"#
        )
    );
    assert_eq!(
        account_output,
        generated_output(
            r#"part of 'account.dart';

const DeepCollectionEquality _deepCollectionEquality = DeepCollectionEquality();
Never _jsonTypeError(Object? value, String key, String expected) => throw ArgumentError.value(value, key, 'expected $expected');
T _jsonAs<T>(Object? value, String key, String expected) => value is T ? value : _jsonTypeError(value, key, expected);
T _jsonParseString<T>(Object? value, String key, String expected, T? Function(String value) parse) => parse(_jsonAs<String>(value, key, 'String')) ?? _jsonTypeError(value, key, expected);
List<Object?> _jsonAsList(Object? value, String key) => _jsonAs<List>(value, key, 'List<Object?>').cast<Object?>();

Map<String, Object?> _jsonAsMap(Object? value, String key) {
  final map = _jsonAs<Map>(value, key, 'Map<String, Object?>');
  try {
    return Map<String, Object?>.from(map);
  } on TypeError {
    _jsonTypeError(value, key, 'Map<String, Object?>');
  }
}
DateTime _jsonAsDateTime(Object? value, String key) => _jsonParseString(value, key, 'ISO-8601 DateTime string', DateTime.tryParse);
Uri _jsonAsUri(Object? value, String key) => _jsonParseString(value, key, 'Uri string', Uri.tryParse);
BigInt _jsonAsBigInt(Object? value, String key) => _jsonParseString(value, key, 'BigInt string', BigInt.tryParse);
T _jsonDecodeWithCodec<T>(dynamic codec, Object? value, String key) {
  if (value == null) {
    throw ArgumentError.value(value, key, 'expected value for SerDeCodec');
  }
  try {
    return codec.deserialize(value as dynamic) as T;
  } catch (error) {
    throw ArgumentError.value(value, key, 'failed SerDeCodec decode: $error');
  }
}

mixin _$Account {
  Map<String, Object?> toJson() => _$AccountToJson(this as Account);
}

Map<String, Object?> _$AccountToJson(Account instance) {
  return <String, Object?>{
    'profile': instance.profile.toJson(),
    'metrics': instance.metrics.map((key, value) => MapEntry(key, value.map((item) => item).toList())),
    'archived': instance.archived,
  };
}
// factory Account.fromJson(Map<String, Object?> json) => _$AccountFromJson(json);
Account _$AccountFromJson(Map<String, Object?> json) {
  final profileValue = Profile.fromJson(_jsonAsMap(json['profile'], 'profile'));
  final metricsValue = _jsonAsMap(json['metrics'], 'metrics').map((mapKey, value) => MapEntry(mapKey, _jsonAsList(value, 'metrics').map((item) => _jsonAs<int>(item, 'metrics', 'int')).toList()));
  final archivedValue = _jsonAs<bool>(json['archived'], 'archived', 'bool');

  return Account(
    profile: profileValue,
    metrics: metricsValue,
    archived: archivedValue,
  );
}
"#
        )
    );
}

#[test]
fn build_writes_custom_serde_codec_outputs() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/audit.dart"),
        "part 'audit.g.dart';\n\
         final class UnixEpochDateTimeCodec implements SerDeCodec<DateTime, int> {\n\
           const UnixEpochDateTimeCodec();\n\
           @override\n\
           int serialize(DateTime value) => value.millisecondsSinceEpoch;\n\
           @override\n\
           DateTime deserialize(int value) => DateTime.fromMillisecondsSinceEpoch(value, isUtc: true);\n\
         }\n\
         const unixEpochDateTimeCodec = UnixEpochDateTimeCodec();\n\
         @Derive([Serialize(), Deserialize()])\n\
         class Audit {\n\
           const Audit({required this.createdAt, this.updatedAt});\n\
           @SerDe(using: unixEpochDateTimeCodec)\n\
           final DateTime createdAt;\n\
           @SerDe(using: unixEpochDateTimeCodec)\n\
           final DateTime? updatedAt;\n\
           factory Audit.fromJson(Map<String, Object?> json) => _$AuditFromJson(json);\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });

    let output = fs::read_to_string(workspace.path().join("lib/audit.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_eq!(
        output,
        generated_output(
            r#"part of 'audit.dart';

Never _jsonTypeError(Object? value, String key, String expected) => throw ArgumentError.value(value, key, 'expected $expected');
T _jsonAs<T>(Object? value, String key, String expected) => value is T ? value : _jsonTypeError(value, key, expected);
T _jsonParseString<T>(Object? value, String key, String expected, T? Function(String value) parse) => parse(_jsonAs<String>(value, key, 'String')) ?? _jsonTypeError(value, key, expected);
List<Object?> _jsonAsList(Object? value, String key) => _jsonAs<List>(value, key, 'List<Object?>').cast<Object?>();

Map<String, Object?> _jsonAsMap(Object? value, String key) {
  final map = _jsonAs<Map>(value, key, 'Map<String, Object?>');
  try {
    return Map<String, Object?>.from(map);
  } on TypeError {
    _jsonTypeError(value, key, 'Map<String, Object?>');
  }
}
DateTime _jsonAsDateTime(Object? value, String key) => _jsonParseString(value, key, 'ISO-8601 DateTime string', DateTime.tryParse);
Uri _jsonAsUri(Object? value, String key) => _jsonParseString(value, key, 'Uri string', Uri.tryParse);
BigInt _jsonAsBigInt(Object? value, String key) => _jsonParseString(value, key, 'BigInt string', BigInt.tryParse);
T _jsonDecodeWithCodec<T>(dynamic codec, Object? value, String key) {
  if (value == null) {
    throw ArgumentError.value(value, key, 'expected value for SerDeCodec');
  }
  try {
    return codec.deserialize(value as dynamic) as T;
  } catch (error) {
    throw ArgumentError.value(value, key, 'failed SerDeCodec decode: $error');
  }
}

mixin _$Audit {
  Map<String, Object?> toJson() => _$AuditToJson(this as Audit);
}

Map<String, Object?> _$AuditToJson(Audit instance) {
  return <String, Object?>{
    'createdAt': unixEpochDateTimeCodec.serialize(instance.createdAt),
    'updatedAt': instance.updatedAt == null
                 ? null
                 : unixEpochDateTimeCodec.serialize(instance.updatedAt!),
  };
}
// factory Audit.fromJson(Map<String, Object?> json) => _$AuditFromJson(json);
Audit _$AuditFromJson(Map<String, Object?> json) {
  final createdAtValue = _jsonDecodeWithCodec<DateTime>(unixEpochDateTimeCodec, json['createdAt'], 'createdAt');
  final updatedAtValue = json['updatedAt'] == null
                         ? null
                         : _jsonDecodeWithCodec<DateTime>(unixEpochDateTimeCodec, json['updatedAt'], 'updatedAt');

  return Audit(
    createdAt: createdAtValue,
    updatedAt: updatedAtValue,
  );
}
"#
        )
    );
}

#[test]
fn build_rejects_invalid_serde_using_values() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/audit.dart"),
        "part 'audit.g.dart';\n\
         @Derive([Serialize(), Deserialize()])\n\
         class Audit {\n\
           const Audit({required this.createdAt});\n\
           @SerDe(using: DateTimeCodec)\n\
           final DateTime createdAt;\n\
           factory Audit.fromJson(Map<String, Object?> json) => _$AuditFromJson(json);\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });

    assert!(result.has_errors());
    assert!(result.diagnostic_files.is_empty());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains(
            "field `createdAt` uses suspicious `SerDe(using: ...)` type reference `DateTimeCodec`",
        )
    }));
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.notes.iter().any(|note| {
            note.contains("Use a codec object such as `const UnixEpochDateTimeCodec()`")
        })
    }));
}

#[test]
fn build_keeps_source_context_for_labeled_diagnostics() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @Derive([ToString(), UnknownTrait()])\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });

    assert_eq!(result.diagnostic_files.len(), 1);
    let file = &result.diagnostic_files[0];
    assert_eq!(file.path, workspace.path().join("lib/user.dart"));
    assert_eq!(file.file_id, result.diagnostics[0].labels[0].file_id);
    assert!(
        file.source_text()
            .contains("@Derive([ToString(), UnknownTrait()])")
    );

    let rendered = render_to_string_with_files(&result.diagnostics[0], &[file.render_context()]);
    assert!(rendered.contains(&format!("  --> {}:2:1", file.path.display())));
    assert!(rendered.contains("2 | @Derive([ToString(), UnknownTrait()])"));
    assert!(rendered.contains("annotation member is not owned by any registered symbol"));
}
