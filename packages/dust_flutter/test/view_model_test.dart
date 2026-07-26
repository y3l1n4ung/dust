import 'dart:async';

import 'package:dust_flutter/state.dart';
import 'package:flutter_test/flutter_test.dart';

final class TestArgs extends ViewModelArgs {
  const TestArgs({super.observer});
}

final class CounterViewModel extends ViewModelBase<int, TestArgs> {
  CounterViewModel({StateObserver? observer})
      : super(TestArgs(observer: observer), initialState: 0);

  static const Object testAction = Object();

  void setCount(int next) {
    emit(next);
  }

  void emitTestEffect(Object effect) {
    emitEffect(effect);
  }

  ViewModelActionToken beginTestAction() {
    return beginAction(testAction);
  }

  bool isCurrentTestAction(ViewModelActionToken token) {
    return isCurrentAction(token);
  }

  Future<bool> runTestAction(
    Future<int> Function() run, {
    void Function(Object error, StackTrace stackTrace)? onError,
  }) {
    return runAction<int>(
      testAction,
      onStart: () => emit(-1),
      run: run,
      onSuccess: emit,
      onError: onError,
    );
  }
}

final class InitViewModel extends ViewModelBase<int, TestArgs> {
  InitViewModel() : super(const TestArgs(), initialState: 0);

  final initCompleters = <Completer<void>>[];
  var initCalls = 0;

  @override
  Future<void> onInit() {
    initCalls += 1;
    final completer = Completer<void>();
    initCompleters.add(completer);
    return completer.future;
  }
}

final class RecordingStateObserver implements StateObserver {
  final effects = <Object>[];

  @override
  void onChanged(Object viewModel, Object? previous, Object? next) {}

  @override
  void onEffect(Object viewModel, Object effect) {
    effects.add(effect);
  }
}

void main() {
  test('invalidateSelf resets sync state to initial state', () {
    final viewModel = CounterViewModel()
      ..setCount(3)
      ..invalidateSelf();

    expect(viewModel.state, 0);
  });

  test('invalidateSelf keeps old action tokens stale after action restarts',
      () {
    final viewModel = CounterViewModel();
    final oldToken = viewModel.beginTestAction();

    viewModel
      ..setCount(3)
      ..invalidateSelf();

    final newToken = viewModel.beginTestAction();

    expect(viewModel.isCurrentTestAction(oldToken), isFalse);
    expect(viewModel.isCurrentTestAction(newToken), isTrue);
  });

  test('runAction ignores stale success result', () async {
    final viewModel = CounterViewModel();
    final firstCompleter = Completer<int>();
    final secondCompleter = Completer<int>();

    final first = viewModel.runTestAction(() => firstCompleter.future);
    expect(viewModel.state, -1);

    final second = viewModel.runTestAction(() => secondCompleter.future);
    expect(viewModel.state, -1);

    secondCompleter.complete(2);
    expect(await second, isTrue);
    expect(viewModel.state, 2);

    firstCompleter.complete(1);
    expect(await first, isFalse);
    expect(viewModel.state, 2);
  });

  test('runAction ignores stale error result', () async {
    final viewModel = CounterViewModel();
    final firstCompleter = Completer<int>();
    final secondCompleter = Completer<int>();

    final first = viewModel.runTestAction(
      () => firstCompleter.future,
      onError: (error, stackTrace) {
        fail('stale errors must not call onError: $error $stackTrace');
      },
    );
    final second = viewModel.runTestAction(() => secondCompleter.future);

    secondCompleter.complete(2);
    expect(await second, isTrue);

    firstCompleter.completeError(StateError('stale'));
    expect(await first, isFalse);
    expect(viewModel.state, 2);
  });

  test('runAction handles current error', () async {
    final viewModel = CounterViewModel();
    final completer = Completer<int>();

    final action = viewModel.runTestAction(
      () => completer.future,
      onError: (error, _) {
        expect(error, isA<StateError>());
        viewModel.setCount(-2);
      },
    );

    completer.completeError(StateError('failed'));

    expect(await action, isTrue);
    expect(viewModel.state, -2);
  });

  test('init runs onInit once for concurrent calls', () async {
    final viewModel = InitViewModel();

    final first = viewModel.init();
    final second = viewModel.init();

    expect(viewModel.initCalls, 1);

    viewModel.initCompleters.single.complete();
    await Future.wait([first, second]);

    await viewModel.init();

    expect(viewModel.initCalls, 1);
  });

  test('init retries after onInit failure', () async {
    final viewModel = InitViewModel();

    final first = viewModel.init();
    viewModel.initCompleters.single.completeError(StateError('failed'));

    await expectLater(first, throwsA(isA<StateError>()));
    expect(viewModel.initCalls, 1);

    final second = viewModel.init();

    expect(viewModel.initCalls, 2);

    viewModel.initCompleters.last.complete();
    await second;

    await viewModel.init();

    expect(viewModel.initCalls, 2);
  });

  test('emitEffect delivers raw effect to stream and observer', () async {
    final observer = RecordingStateObserver();
    final viewModel = CounterViewModel(observer: observer);
    final effects = <Object>[];
    final subscription = viewModel.effects.listen(effects.add);

    addTearDown(viewModel.dispose);
    addTearDown(subscription.cancel);

    final effect = Object();
    viewModel.emitTestEffect(effect);

    expect(observer.effects.single, same(effect));
    expect(effects, isEmpty);

    await pumpEventQueue();

    expect(effects.single, same(effect));
  });

  test('effects emitted before subscription are not replayed', () async {
    final viewModel = CounterViewModel();
    final effects = <Object>[];

    addTearDown(viewModel.dispose);

    viewModel.emitTestEffect('early');

    final subscription = viewModel.effects.listen(effects.add);
    addTearDown(subscription.cancel);

    await pumpEventQueue();

    expect(effects, isEmpty);
  });

  test('effects are dropped when subscription cancels before delivery',
      () async {
    final viewModel = CounterViewModel();
    final effects = <Object>[];
    final subscription = viewModel.effects.listen(effects.add);

    addTearDown(viewModel.dispose);

    viewModel.emitTestEffect('toast');
    await subscription.cancel();
    await pumpEventQueue();

    expect(effects, isEmpty);
  });

  test('emitEffect unwraps deprecated StateEffect wrapper', () async {
    final observer = RecordingStateObserver();
    final viewModel = CounterViewModel(observer: observer);
    final effects = <Object>[];
    final subscription = viewModel.effects.listen(effects.add);

    addTearDown(viewModel.dispose);
    addTearDown(subscription.cancel);

    final effect = Object();
    // ignore: deprecated_member_use_from_same_package
    viewModel.emitTestEffect(StateEffect(effect));
    await pumpEventQueue();

    expect(effects.single, same(effect));
    expect(observer.effects.single, same(effect));
  });
}
