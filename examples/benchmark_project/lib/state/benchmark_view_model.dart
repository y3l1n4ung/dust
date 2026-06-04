import 'package:dust_flutter/state.dart';

import 'benchmark_state.dart';

part 'benchmark_view_model.g.dart';

final class BenchmarkViewModelArgs extends ViewModelArgs {
  const BenchmarkViewModelArgs({super.observer});
}

@ViewModel(state: BenchmarkState, args: BenchmarkViewModelArgs)
class BenchmarkViewModel extends $BenchmarkViewModel {
  BenchmarkViewModel(super.args);

  void selectFeature(String feature) {
    emit(state.copyWith(activeFeature: feature));
  }

  void recordBuild(BenchmarkMode mode) {
    emit(state.copyWith(mode: mode, buildsRun: state.buildsRun + 1));
    emitEffect('benchmark:${mode.name}:${state.buildsRun}');
  }
}
