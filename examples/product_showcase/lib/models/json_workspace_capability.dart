import 'package:dust_dart/serde.dart';

part 'json_workspace_capability.g.dart';

@Derive([Serialize(), Deserialize()])
enum JsonWorkspaceKind { retail, wholesale }

final class JsonWorkspaceProfile {
  const JsonWorkspaceProfile({
    required this.id,
    required this.kind,
  });

  factory JsonWorkspaceProfile.fromJson(Map<String, Object?> json) {
    return JsonWorkspaceProfile(
      id: json['id'] as String,
      kind: JsonWorkspaceKind.values.byName(json['kind'] as String),
    );
  }

  final String id;
  final JsonWorkspaceKind kind;

  Map<String, Object?> toJson() => {
        'id': id,
        'kind': kind.name,
      };
}

@Derive([Serialize(), Deserialize()])
class JsonWorkspaceAccount with _$JsonWorkspaceAccount {
  const JsonWorkspaceAccount({
    required this.profile,
    required this.active,
  });

  factory JsonWorkspaceAccount.fromJson(Map<String, Object?> json) =>
      _$JsonWorkspaceAccountFromJson(json);

  final JsonWorkspaceProfile profile;
  final bool active;
}
