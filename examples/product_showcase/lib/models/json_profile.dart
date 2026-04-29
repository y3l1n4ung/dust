import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'json_profile.g.dart';

@Derive([ToString(), Eq(), Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)
class JsonProfile with _$JsonProfileDust {
  const JsonProfile({
    required this.id,
    this.displayName,
    this.tags = const ['guest'],
  });

  factory JsonProfile.fromJson(Map<String, Object?> json) =>
      _$JsonProfileFromJson(json);

  final String id;

  @SerDe(rename: 'display_name', aliases: ['displayName'])
  final String? displayName;

  @SerDe(defaultValue: ['guest'])
  final List<String> tags;
}
