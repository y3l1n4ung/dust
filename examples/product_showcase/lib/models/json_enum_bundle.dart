import 'package:dust_dart/serde.dart';

part 'json_enum_bundle.g.dart';

/// Access level values for the product showcase example.
@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.kebabCase)
enum AccessLevel {
  /// Super admin access level.
  @SerDe(rename: 'owner')
  superAdmin,

  /// Guest user access level.
  guestUser,

  /// Legacy staff access level.
  @SerDe(skip: true)
  legacyStaff,

  /// Read only access level.
  readOnly,
}

/// Review state values for the product showcase example.
@Derive([Serialize(), Deserialize()])
enum ReviewState {
  /// Pending review state.
  pending,

  /// Approved review state.
  approved,

  /// Archived review state.
  archived,
}

/// JSON enum bundle model for the product showcase example.
@Derive([ToString(), Eq(), Serialize(), Deserialize()])
class JsonEnumBundle with _$JsonEnumBundle {
  /// Creates a [JsonEnumBundle].
  const JsonEnumBundle({
    required this.primaryLevel,
    required this.fallbackState,
    required this.levels,
    required this.stateByRegion,
    required this.states,
  });

  /// Creates a [JsonEnumBundle] from JSON.
  factory JsonEnumBundle.fromJson(Map<String, Object?> json) =>
      _$JsonEnumBundleFromJson(json);

  /// Primary level.
  @SerDe(rename: 'primary_level', aliases: ['primaryLevel'])
  final AccessLevel primaryLevel;

  /// Fallback state.
  final ReviewState? fallbackState;

  /// Levels.
  final List<AccessLevel> levels;

  /// State by region.
  final Map<String, ReviewState> stateByRegion;

  /// States.
  final Set<ReviewState> states;
}
