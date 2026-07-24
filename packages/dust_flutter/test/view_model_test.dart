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
    await pumpEventQueue();

    expect(effects.single, same(effect));
    expect(observer.effects.single, same(effect));
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
