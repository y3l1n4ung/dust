import 'package:dust_dart/serde.dart';

part 'benchmark_state.g.dart';

/// Benchmark mode values for the benchmark example.
@Derive([Serialize(), Deserialize()])
enum BenchmarkMode {
  /// Cold benchmark mode.
  @SerDe(rename: 'cold-start')
  cold,

  /// Warm benchmark mode.
  warm,

  /// Invalidated benchmark mode.
  @SerDe(skip: true)
  invalidated,
}

/// Benchmark state for the benchmark example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class BenchmarkState with _$BenchmarkState {
  /// Creates a [BenchmarkState].
  const BenchmarkState({
    this.mode = BenchmarkMode.warm,
    this.activeFeature = 'derive',
    this.buildsRun = 0,
  });

  /// Mode.
  final BenchmarkMode mode;

  /// Active feature.
  final String activeFeature;

  /// Builds run.
  final int buildsRun;

  /// Creates a [BenchmarkState] from JSON.
  factory BenchmarkState.fromJson(Map<String, Object?> json) =>
      _$BenchmarkStateFromJson(json);
}
