import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'json_serde_options.g.dart';

@Derive([ToString(), Eq(), Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)
class JsonSerdeOptions with _$JsonSerdeOptionsDust {
  const JsonSerdeOptions({
    required this.id,
    required this.displayName,
    this.tags = const ['guest'],
    this.serverOnly = 'server-default',
    this.clientOnly = 'client-default',
    this.hidden = 'hidden-default',
  });

  factory JsonSerdeOptions.fromJson(Map<String, Object?> json) =>
      _$JsonSerdeOptionsFromJson(json);

  final String id;

  @SerDe(rename: 'display_name', aliases: ['displayName'])
  final String displayName;

  @SerDe(defaultValue: ['guest'])
  final List<String> tags;

  @SerDe(skipSerializing: true, defaultValue: 'server-default')
  final String serverOnly;

  @SerDe(skipDeserializing: true, defaultValue: 'client-default')
  final String clientOnly;

  @SerDe(skip: true, defaultValue: 'hidden-default')
  final String hidden;
}
