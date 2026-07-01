import 'package:dust_flutter/state.dart';
import 'package:flutter_test/flutter_test.dart';

final class TestArgs extends ViewModelArgs {
  const TestArgs();
}

final class CounterViewModel extends ViewModelBase<int, TestArgs> {
  CounterViewModel() : super(const TestArgs(), initialState: 0);

  void setCount(int next) {
    emit(next);
  }
}

void main() {
  test('invalidateSelf resets sync state to initial state', () {
    final viewModel = CounterViewModel()
      ..setCount(3)
      ..invalidateSelf();

    expect(viewModel.state, 0);
  });
}
