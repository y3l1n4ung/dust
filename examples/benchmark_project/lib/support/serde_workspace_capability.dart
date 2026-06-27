import 'package:dust_dart/serde.dart';

part 'serde_workspace_capability.g.dart';

@Derive([Serialize(), Deserialize()])
enum BenchmarkWorkspaceKind { primary, fallback }

final class BenchmarkPage<T> {
  const BenchmarkPage({required this.items, required this.total});

  final List<T> items;
  final int total;
}

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

final class BenchmarkWorkspaceProfilePageCodec
    implements
        SerDeCodec<BenchmarkPage<BenchmarkWorkspaceProfile>,
            Map<String, Object?>> {
  const BenchmarkWorkspaceProfilePageCodec();

  @override
  Map<String, Object?> serialize(
    BenchmarkPage<BenchmarkWorkspaceProfile> value,
  ) =>
      {
        'items': value.items.map((item) => item.toJson()).toList(),
        'total': value.total,
      };

  @override
  BenchmarkPage<BenchmarkWorkspaceProfile> deserialize(
    Map<String, Object?> value,
  ) =>
      BenchmarkPage(
        items: JsonHelper.decodeList(
          value['items'],
          'items',
          (item, key) =>
              BenchmarkWorkspaceProfile.fromJson(JsonHelper.asMap(item, key)),
        ),
        total: JsonHelper.as<int>(value['total'], 'total', 'int'),
      );
}

const benchmarkWorkspaceProfilePageCodec = BenchmarkWorkspaceProfilePageCodec();

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

@Derive([Serialize(), Deserialize()])
class BenchmarkWorkspacePageEnvelope with _$BenchmarkWorkspacePageEnvelope {
  const BenchmarkWorkspacePageEnvelope({required this.page});

  factory BenchmarkWorkspacePageEnvelope.fromJson(Map<String, Object?> json) =>
      _$BenchmarkWorkspacePageEnvelopeFromJson(json);

  @SerDe(using: benchmarkWorkspaceProfilePageCodec)
  final BenchmarkPage<BenchmarkWorkspaceProfile> page;
}
