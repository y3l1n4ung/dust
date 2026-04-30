import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'json_enum_bundle.g.dart';

@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.kebabCase)
enum AccessLevel {
  superAdmin,
  guestUser,
  readOnly,
}

@Derive([Serialize(), Deserialize()])
enum ReviewState {
  pending,
  approved,
  archived,
}

@Derive([ToString(), Eq(), Serialize(), Deserialize()])
class JsonEnumBundle with _$JsonEnumBundleDust {
  const JsonEnumBundle({
    required this.primaryLevel,
    required this.fallbackState,
    required this.levels,
    required this.stateByRegion,
    required this.states,
  });

  factory JsonEnumBundle.fromJson(Map<String, Object?> json) =>
      _$JsonEnumBundleFromJson(json);

  @SerDe(rename: 'primary_level', aliases: ['primaryLevel'])
  final AccessLevel primaryLevel;

  final ReviewState? fallbackState;
  final List<AccessLevel> levels;
  final Map<String, ReviewState> stateByRegion;
  final Set<ReviewState> states;
}
