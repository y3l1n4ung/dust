import 'package:dust_dart/serde.dart';

part 'json_serde_options.g.dart';

/// My enum values for the product showcase example.
@Derive([Serialize(), Deserialize()])
enum MyEnum {
  /// A my enum.
  A,

  /// B my enum.
  B,
}

/// JSON serde options model for the product showcase example.
@Derive([ToString(), Eq(), Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)
class JsonSerdeOptions with _$JsonSerdeOptions {
  /// Creates a [JsonSerdeOptions].
  const JsonSerdeOptions({
    required this.id,
    required this.displayName,
    required this.e,
    this.tags = const ['guest'],
    this.serverOnly = 'server-default',
    this.clientOnly = 'client-default',
    this.hidden = 'hidden-default',
  });

  /// Creates a [JsonSerdeOptions] from JSON.
  factory JsonSerdeOptions.fromJson(Map<String, Object?> json) =>
      _$JsonSerdeOptionsFromJson(json);

  /// Unique identifier.
  final String id;

  /// E.
  final MyEnum e;

  /// Display name.
  @SerDe(rename: 'display_name', aliases: ['displayName'])
  final String displayName;

  /// Tags.
  @SerDe(defaultValue: ['guest'])
  final List<String> tags;

  /// Server only.
  @SerDe(skipSerializing: true, defaultValue: 'server-default')
  final String serverOnly;

  /// Client only.
  @SerDe(skipDeserializing: true, defaultValue: 'client-default')
  final String clientOnly;

  /// Hidden.
  @SerDe(skip: true, defaultValue: 'hidden-default')
  final String hidden;
}
