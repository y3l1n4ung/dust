import 'package:dust_dart/serde.dart';

part 'json_profile.g.dart';

/// JSON profile model for the product showcase example.
@Derive([ToString(), Eq(), Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)
class JsonProfile with _$JsonProfile {
  /// Creates a [JsonProfile].
  const JsonProfile({
    required this.id,
    this.displayName,
    this.tags = const ['guest'],
  });

  /// Creates a [JsonProfile] from JSON.
  factory JsonProfile.fromJson(Map<String, Object?> json) =>
      _$JsonProfileFromJson(json);

  /// Unique identifier.
  final String id;

  /// Display name.
  @SerDe(rename: 'display_name', aliases: ['displayName'])
  final String? displayName;

  /// Tags.
  @SerDe(defaultValue: ['guest'])
  final List<String> tags;
}
