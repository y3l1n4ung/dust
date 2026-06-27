import 'package:dust_dart/serde.dart';

part 'serde_workspace_capability.g.dart';

@Derive([Serialize(), Deserialize()])
enum BenchmarkWorkspaceKind { primary, fallback }

final class BenchmarkWorkspaceProfile {
  const BenchmarkWorkspaceProfile({
    required this.id,
    required this.kind,
  });

  factory BenchmarkWorkspaceProfile.fromJson(Map<String, Object?> json) {
    return BenchmarkWorkspaceProfile(
      id: json['id'] as String,
      kind: BenchmarkWorkspaceKind.values.byName(json['kind'] as String),
    );
  }

  final String id;
  final BenchmarkWorkspaceKind kind;

  Map<String, Object?> toJson() => {
        'id': id,
        'kind': kind.name,
      };
}

@Derive([Serialize(), Deserialize()])
class BenchmarkWorkspaceAccount with _$BenchmarkWorkspaceAccount {
  const BenchmarkWorkspaceAccount({
    required this.profile,
    required this.score,
  });

  factory BenchmarkWorkspaceAccount.fromJson(Map<String, Object?> json) =>
      _$BenchmarkWorkspaceAccountFromJson(json);

  final BenchmarkWorkspaceProfile profile;
  final int score;
}
