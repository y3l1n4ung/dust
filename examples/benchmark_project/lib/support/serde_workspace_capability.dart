import 'package:dust_dart/serde.dart';

part 'serde_workspace_capability.g.dart';

/// Benchmark workspace kind values for the benchmark example.
@Derive([Serialize(), Deserialize()])
enum BenchmarkWorkspaceKind {
  /// Primary benchmark workspace kind.
  primary,

  /// Fallback benchmark workspace kind.
  fallback,
}

/// Benchmark page.
final class BenchmarkPage<T> {
  /// Creates a [BenchmarkPage].
  const BenchmarkPage({required this.items, required this.total});

  /// Items.
  final List<T> items;

  /// Total.
  final int total;
}

/// Benchmark workspace profile model for the benchmark example.
final class BenchmarkWorkspaceProfile {
  /// Creates a [BenchmarkWorkspaceProfile].
  const BenchmarkWorkspaceProfile({
    required this.id,
    required this.kind,
  });

  /// Creates a [BenchmarkWorkspaceProfile] from JSON.
  factory BenchmarkWorkspaceProfile.fromJson(Map<String, Object?> json) {
    return BenchmarkWorkspaceProfile(
      id: json['id'] as String,
      kind: BenchmarkWorkspaceKind.values.byName(json['kind'] as String),
    );
  }

  /// Unique identifier.
  final String id;

  /// Kind.
  final BenchmarkWorkspaceKind kind;

  /// To JSON benchmark workspace kind.
  Map<String, Object?> toJson() => {
        'id': id,
        'kind': kind.name,
      };
}

/// Benchmark workspace profile page codec model for the benchmark example.
final class BenchmarkWorkspaceProfilePageCodec
    implements
        SerDeCodec<BenchmarkPage<BenchmarkWorkspaceProfile>,
            Map<String, Object?>> {
  /// Creates a [BenchmarkWorkspaceProfilePageCodec].
  const BenchmarkWorkspaceProfilePageCodec();

  @override
  Map<String, Object?> serialize(
    BenchmarkPage<BenchmarkWorkspaceProfile> value,
  ) =>
      {
        'items': value.items.map((item) => item.toJson()).toList(),
        'total': value.total,
      };

  /// Benchmark workspace profile page codec.
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

/// Benchmark workspace profile page codec.
const benchmarkWorkspaceProfilePageCodec = BenchmarkWorkspaceProfilePageCodec();

/// Benchmark workspace account model for the benchmark example.
@Derive([Serialize(), Deserialize()])
class BenchmarkWorkspaceAccount with _$BenchmarkWorkspaceAccount {
  /// Creates a [BenchmarkWorkspaceAccount].
  const BenchmarkWorkspaceAccount({
    required this.profile,
    required this.score,
  });

  /// Creates a [BenchmarkWorkspaceAccount] from JSON.
  factory BenchmarkWorkspaceAccount.fromJson(Map<String, Object?> json) =>
      _$BenchmarkWorkspaceAccountFromJson(json);

  /// Profile.
  final BenchmarkWorkspaceProfile profile;

  /// Score.
  final int score;
}

/// Benchmark workspace page envelope model for the benchmark example.
@Derive([Serialize(), Deserialize()])
class BenchmarkWorkspacePageEnvelope with _$BenchmarkWorkspacePageEnvelope {
  /// Creates a [BenchmarkWorkspacePageEnvelope].
  const BenchmarkWorkspacePageEnvelope({required this.page});

  /// Creates a [BenchmarkWorkspacePageEnvelope] from JSON.
  factory BenchmarkWorkspacePageEnvelope.fromJson(Map<String, Object?> json) =>
      _$BenchmarkWorkspacePageEnvelopeFromJson(json);

  /// Page.
  @SerDe(using: benchmarkWorkspaceProfilePageCodec)
  final BenchmarkPage<BenchmarkWorkspaceProfile> page;
}
