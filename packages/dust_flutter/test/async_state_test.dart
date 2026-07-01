import 'dart:async';

import 'package:dust_flutter/state.dart';
import 'package:flutter_test/flutter_test.dart';

final class TestArgs extends ViewModelArgs {
  const TestArgs();
}

final class TestAsyncViewModel extends AsyncViewModelBase<int, TestArgs> {
  TestAsyncViewModel() : super(const TestArgs());

  final loads = <Completer<int>>[];

  @override
  Future<int> loadData() {
    final completer = Completer<int>();
    loads.add(completer);
    return completer.future;
  }
}

final class NullableAsyncViewModel extends AsyncViewModelBase<int?, TestArgs> {
  NullableAsyncViewModel() : super(const TestArgs());

  bool shouldFail = false;

  @override
  Future<int?> loadData() async {
    if (shouldFail) {
      throw StateError('failed');
    }
    return null;
  }
}

void main() {
  test('load moves initial to data', () async {
    final viewModel = TestAsyncViewModel();

    final load = viewModel.load();
    expect(viewModel.state, isA<AsyncLoading<int>>());

    viewModel.loads.single.complete(7);
    await load;

    expect(viewModel.state, isA<AsyncData<int>>());
    expect(viewModel.data, 7);
    expect(viewModel.visibleData, 7);
  });

  test('refresh preserves previous data on error', () async {
    final viewModel = TestAsyncViewModel();

    final load = viewModel.load();
    viewModel.loads.single.complete(7);
    await load;

    final refresh = viewModel.refresh();
    expect(viewModel.state, isA<AsyncLoading<int>>());
    expect(viewModel.state.isRefreshing, isTrue);
    expect(viewModel.state.hasPreviousData, isTrue);
    expect(viewModel.state.data, 7);

    viewModel.loads.last.completeError(StateError('failed'));
    await refresh;

    expect(viewModel.state, isA<AsyncError<int>>());
    expect(viewModel.state.hasPreviousData, isTrue);
    expect(viewModel.state.data, 7);
    expect(viewModel.state.previousData, 7);
    expect(viewModel.state.error, isA<StateError>());
  });

  test('stale load result is ignored', () async {
    final viewModel = TestAsyncViewModel();

    final first = viewModel.load();
    final second = viewModel.refresh();

    viewModel.loads[1].complete(2);
    await second;

    expect(viewModel.data, 2);

    viewModel.loads[0].complete(1);
    await first;

    expect(viewModel.data, 2);
  });

  test('nullable data is still present data', () async {
    final viewModel = NullableAsyncViewModel();

    await viewModel.load();

    expect(viewModel.state, isA<AsyncData<int?>>());
    expect(viewModel.state.hasData, isTrue);
    expect(viewModel.data, isNull);

    viewModel.shouldFail = true;
    await viewModel.refresh();

    expect(viewModel.state, isA<AsyncError<int?>>());
    expect(viewModel.state.hasPreviousData, isTrue);
    expect(viewModel.state.previousData, isNull);
  });
}
