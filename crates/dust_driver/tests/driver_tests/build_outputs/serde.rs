use std::fs;

use dust_driver::{BuildRequest, run_build};

use crate::support::{generated_output, make_workspace, write_file};

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
        db: Default::default(),
    });

    let profile_output = fs::read_to_string(workspace.path().join("lib/profile.g.dart")).unwrap();
    let account_output = fs::read_to_string(workspace.path().join("lib/account.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_eq!(
        profile_output,
        generated_output(
            r#"part of 'profile.dart';

const DeepCollectionEquality _deepCollectionEquality = DeepCollectionEquality();

mixin _$Profile {
  Map<String, Object?> toJson() => _$ProfileToJson(this as Profile);
}

Map<String, Object?> _$ProfileToJson(Profile instance) {
  return <String, Object?>{
    'id': instance.id,
    'display_name': instance.displayName,
    'tags': instance.tags
        .map((item) => item)
        .toList(),
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

  final idValue = JsonHelper.as<String>(json['id'], 'id', 'String');
  var rawDisplayNameKey = 'display_name';
  Object? rawDisplayName;
  if (json.containsKey('display_name')) {
    rawDisplayName = json['display_name'];
  } else if (json.containsKey('displayName')) {
    rawDisplayNameKey = 'displayName';
    rawDisplayName = json['displayName'];
  }
  final displayNameValue = rawDisplayName == null
      ? null
      : JsonHelper.as<String>(rawDisplayName, rawDisplayNameKey, 'String');
  final tagsValue = json.containsKey('tags')
      ? JsonHelper.asList(json['tags'], 'tags')
      .map((item) => JsonHelper.as<String>(item, 'tags', 'String'))
      .toList()
      : const ['guest'];

  return Profile(id: idValue, displayName: displayNameValue, tags: tagsValue);
}
"#
        )
    );
    assert_eq!(
        account_output,
        generated_output(
            r#"part of 'account.dart';

const DeepCollectionEquality _deepCollectionEquality = DeepCollectionEquality();

mixin _$Account {
  Map<String, Object?> toJson() => _$AccountToJson(this as Account);
}

Map<String, Object?> _$AccountToJson(Account instance) {
  return <String, Object?>{
    'profile': instance.profile.toJson(),
    'metrics': instance.metrics
        .map(
          (key, value) => MapEntry(
            key,
            value
                .map((item) => item)
                .toList(),
          ),
        ),
    'archived': instance.archived,
  };
}
// factory Account.fromJson(Map<String, Object?> json) => _$AccountFromJson(json);
Account _$AccountFromJson(Map<String, Object?> json) {
  final profileValue = Profile.fromJson(JsonHelper.asMap(json['profile'], 'profile'));
  final metricsValue = JsonHelper.asMap(json['metrics'], 'metrics')
      .map(
        (mapKey, value) => MapEntry(
          mapKey,
          JsonHelper.asList(value, 'metrics')
              .map((item) => JsonHelper.as<int>(item, 'metrics', 'int'))
              .toList(),
        ),
      );
  final archivedValue = JsonHelper.as<bool>(
    json['archived'],
    'archived',
    'bool',
  );

  return Account(profile: profileValue, metrics: metricsValue, archived: archivedValue);
}
"#
        )
    );
}
