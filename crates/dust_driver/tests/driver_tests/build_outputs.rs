use std::fs;

use dust_driver::{BuildRequest, run_build};

use super::support::{make_workspace, write_file};

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
    assert!(models_output.contains("part of 'models.dart';"));
    assert!(models_output.contains("mixin _$UserDust {"));
    assert!(models_output.contains("User get _dustSelf => this as User;"));
    assert!(models_output.contains("String toString() {\n    return 'User('"));
    assert!(models_output.contains("'id: ${_dustSelf.id}, '"));
    assert!(models_output.contains("'age: ${_dustSelf.age}'"));
    assert!(models_output.contains("mixin _$TeamDust {"));
    assert!(models_output.contains("Team copyWith({"));
    assert!(models_output.contains("String? name,"));
    assert!(models_output.contains("name ?? _dustSelf.name,"));
    assert!(request_output.contains("part of 'request.dart';"));
    assert!(request_output.contains("mixin _$RequestDust {"));
    assert!(request_output.contains("Request copyWith({"));
    assert!(request_output.contains("String? path,"));
    assert!(request_output.contains("Map<String, String>? headers,"));
    assert!(!request_output.contains("final nextPathSource = path ?? _dustSelf.path;"));
    assert!(!request_output.contains("final nextHeadersSource = headers ?? _dustSelf.headers;"));
    assert!(
        request_output
            .contains("final nextHeaders = Map<String, String>.of(headers ?? _dustSelf.headers);")
    );
    assert!(request_output.contains("return Request.create("));
    assert!(request_output.contains("path: path ?? _dustSelf.path,"));
    assert!(request_output.contains("headers: nextHeaders,"));
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
    assert!(
        profile_output.contains("Map<String, Object?> toJson() => _$ProfileToJson(_dustSelf);")
    );
    assert!(profile_output.contains(
        "// factory Profile.fromJson(Map<String, Object?> json) => _$ProfileFromJson(json);"
    ));
    assert!(profile_output.contains("Profile _$ProfileFromJson(Map<String, Object?> json)"));
    assert!(
        profile_output.contains("T _dustJsonAs<T>(Object? value, String key, String expected)")
    );
    assert!(
        profile_output
            .contains("const allowedKeys = <String>{'id', 'display_name', 'displayName', 'tags'};")
    );
    assert!(profile_output.contains(
        "final rawDisplayNameKey = json.containsKey('display_name') ? 'display_name' : json.containsKey('displayName') ? 'displayName' : 'display_name';"
    ));
    assert!(profile_output.contains("final tagsValue = json.containsKey('tags') ?"));
    assert!(profile_output.contains(": const ['guest'];"));
    assert!(
        account_output.contains("Map<String, Object?> toJson() => _$AccountToJson(_dustSelf);")
    );
    assert!(account_output.contains("'profile': instance.profile.toJson()"));
    assert!(
        account_output.contains("Profile.fromJson(_dustJsonAsMap(json['profile'], 'profile'))")
    );
    assert!(account_output.contains("_dustJsonAsMap(json['metrics'], 'metrics').map((mapKey, value) => MapEntry(mapKey, _dustJsonAsList(value, 'metrics').map((item) => _dustJsonAs<int>(item, 'metrics', 'int')).toList()))"));
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
    assert!(output.contains(
        "// factory Audit.fromJson(Map<String, Object?> json) => _$AuditFromJson(json);"
    ));
    assert!(output.contains("'createdAt': unixEpochDateTimeCodec.serialize(instance.createdAt)"));
    assert!(output.contains("'updatedAt': instance.updatedAt == null"));
    assert!(output.contains("unixEpochDateTimeCodec.serialize(instance.updatedAt!)"));
    assert!(output.contains(
        "final createdAtValue = _dustJsonDecodeWithCodec<DateTime>(unixEpochDateTimeCodec, json['createdAt'], 'createdAt');"
    ));
    assert!(output.contains("final updatedAtValue = json['updatedAt'] == null"));
    assert!(output.contains(
        "_dustJsonDecodeWithCodec<DateTime>(unixEpochDateTimeCodec, json['updatedAt'], 'updatedAt')"
    ));
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
