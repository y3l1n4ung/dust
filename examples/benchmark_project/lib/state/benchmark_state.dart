import 'package:dust_dart/serde.dart';

part 'benchmark_state.g.dart';

@Derive([Serialize(), Deserialize()])
enum BenchmarkMode { cold, warm, invalidated }

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class BenchmarkState with _$BenchmarkState {
  const BenchmarkState({
    this.mode = BenchmarkMode.warm,
    this.activeFeature = 'derive',
    this.buildsRun = 0,
  });

  final BenchmarkMode mode;
  final String activeFeature;
  final int buildsRun;

  factory BenchmarkState.fromJson(Map<String, Object?> json) =>
      _$BenchmarkStateFromJson(json);
}
