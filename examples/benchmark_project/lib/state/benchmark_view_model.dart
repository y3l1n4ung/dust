import 'package:dust_flutter/state.dart';

import 'benchmark_state.dart';

part 'benchmark_view_model.g.dart';

/// Benchmark view model args model for the benchmark example.
final class BenchmarkViewModelArgs extends ViewModelArgs {
  /// Creates a [BenchmarkViewModelArgs].
  const BenchmarkViewModelArgs({super.observer});
}

/// Benchmark view model for the benchmark example.
@ViewModel(state: BenchmarkState, args: BenchmarkViewModelArgs)
class BenchmarkViewModel extends $BenchmarkViewModel {
  /// Creates a [BenchmarkViewModel].
  BenchmarkViewModel(super.args);

  /// Selects feature.
  void selectFeature(String feature) {
    emit(state.copyWith(activeFeature: feature));
  }

  /// Records build.
  void recordBuild(BenchmarkMode mode) {
    emit(state.copyWith(mode: mode, buildsRun: state.buildsRun + 1));
    emitEffect('benchmark:${mode.name}:${state.buildsRun}');
  }
}
