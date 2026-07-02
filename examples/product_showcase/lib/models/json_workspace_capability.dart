import 'package:dust_dart/serde.dart';

part 'json_workspace_capability.g.dart';

/// JSON workspace kind values for the product showcase example.
@Derive([Serialize(), Deserialize()])
enum JsonWorkspaceKind {
  /// Retail JSON workspace kind.
  retail,

  /// Wholesale JSON workspace kind.
  wholesale,
}

/// JSON workspace profile model for the product showcase example.
final class JsonWorkspaceProfile {
  /// Creates a [JsonWorkspaceProfile].
  const JsonWorkspaceProfile({
    required this.id,
    required this.kind,
  });

  /// Creates a [JsonWorkspaceProfile] from JSON.
  factory JsonWorkspaceProfile.fromJson(Map<String, Object?> json) {
    return JsonWorkspaceProfile(
      id: json['id'] as String,
      kind: JsonWorkspaceKind.values.byName(json['kind'] as String),
    );
  }

  /// Unique identifier.
  final String id;

  /// Kind.
  final JsonWorkspaceKind kind;

  /// To JSON JSON workspace kind.
  Map<String, Object?> toJson() => {
        'id': id,
        'kind': kind.name,
      };
}

/// JSON workspace account model for the product showcase example.
@Derive([Serialize(), Deserialize()])
class JsonWorkspaceAccount with _$JsonWorkspaceAccount {
  /// Creates a [JsonWorkspaceAccount].
  const JsonWorkspaceAccount({
    required this.profile,
    required this.active,
  });

  /// Creates a [JsonWorkspaceAccount] from JSON.
  factory JsonWorkspaceAccount.fromJson(Map<String, Object?> json) =>
      _$JsonWorkspaceAccountFromJson(json);

  /// Profile.
  final JsonWorkspaceProfile profile;

  /// Active.
  final bool active;
}
