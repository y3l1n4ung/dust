import 'dart:async';

import 'package:dust_state/dust_state.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('emit notifies only when state changes', () {
    final observer = RecordingObserver();
    final vm = CounterViewModel(CounterArgs(observer: observer));
    var notifications = 0;
    vm.addListener(() => notifications += 1);

    vm.setCount(0);
    vm.setCount(1);

    expect(notifications, 1);
    expect(observer.changes, ['0 -> 1']);
  });

  test(
    'emitEffect notifies observer and stream without changing state',
    () async {
      final observer = RecordingObserver();
      final vm = CounterViewModel(CounterArgs(observer: observer));
      final effects = <Object>[];
      final sub = vm.effects.listen(effects.add);

      vm.showToast('saved');
      await Future<void>.delayed(Duration.zero);

      expect(vm.state, 0);
      expect(effects, ['saved']);
      expect(observer.effects, ['saved']);
      await sub.cancel();
    },
  );

  test(
    'StateEffect and SilentStateObserver preserve public no-op contracts',
    () {
      const effect = StateEffect('saved');
      const observer = SilentStateObserver();
      const logging = LoggingStateObserver();
      const annotation = ViewModel(
        state: CounterStateForAnnotation,
        args: CounterArgs,
        initial: 0,
      );

      observer.onChanged(Object(), 1, 2);
      observer.onEffect(Object(), effect);
      logging.onChanged(Object(), 1, 2);
      logging.onEffect(Object(), effect);

      expect(effect.value, 'saved');
      expect(annotation.state, CounterStateForAnnotation);
      expect(annotation.args, CounterArgs);
      expect(annotation.initial, 0);
    },
  );

  test('effects are broadcast to multiple listeners', () async {
    final vm = CounterViewModel(const CounterArgs());
    final first = <Object>[];
    final second = <Object>[];
    final firstSub = vm.effects.listen(first.add);
    final secondSub = vm.effects.listen(second.add);

    vm.showToast('saved');
    await Future<void>.delayed(Duration.zero);

    expect(first, ['saved']);
    expect(second, ['saved']);
    await firstSub.cancel();
    await secondSub.cancel();
  });

  test('init runs once', () async {
    final vm = CounterViewModel(const CounterArgs());
    final bare = BareViewModel(const CounterArgs());

    await Future.wait([vm.init(), vm.init()]);
    await vm.init();
    await bare.init();

    expect(vm.initCalls, 1);
    expect(bare.state, 0);
  });

  test('init retries after failure and then stays complete', () async {
    final vm = CounterViewModel(const CounterArgs())..failNextInit = true;

    await expectLater(vm.init(), throwsStateError);
    await vm.init();
    await vm.init();

    expect(vm.initCalls, 2);
  });

  test('init after dispose is ignored', () async {
    final vm = CounterViewModel(const CounterArgs())..dispose();

    await vm.init();

    expect(vm.initCalls, 0);
  });

  test('stale async actions do not overwrite newer state', () async {
    final vm = CounterViewModel(const CounterArgs());
    final first = Completer<int>();
    final second = Completer<int>();

    final firstRun = vm.loadCount(first);
    final secondRun = vm.loadCount(second);

    second.complete(2);
    await secondRun;
    first.complete(1);
    await firstRun;

    expect(vm.state, 2);
  });

  test('cancelAction invalidates a pending token', () async {
    final vm = CounterViewModel(const CounterArgs());
    final request = Completer<int>();

    final run = vm.loadCount(request);
    vm.cancelLoadCount();
    request.complete(3);
    await run;

    expect(vm.state, 0);
  });

  test('emit and effects are ignored after dispose', () async {
    final vm = CounterViewModel(const CounterArgs())..dispose();

    vm
      ..setCount(9)
      ..showToast('ignored')
      ..dispose();

    expect(vm.state, 0);
  });

  testWidgets(
    'owned scope initializes once and value scope does not initialize',
    (tester) async {
      late CounterViewModel owned;
      await tester.pumpWidget(
        Directionality(
          textDirection: TextDirection.ltr,
          child: ViewModelOwner<CounterViewModel, CounterArgs>(
            args: (_) => const CounterArgs(),
            create: (_, args) => owned = CounterViewModel(args),
            builder: (_, __) => const SizedBox(),
          ),
        ),
      );
      await tester.pump();

      expect(owned.initCalls, 1);

      final external = CounterViewModel(const CounterArgs());
      await tester.pumpWidget(
        Directionality(
          textDirection: TextDirection.ltr,
          child: ViewModelOwner<CounterViewModel, CounterArgs>.value(
            value: external,
            builder: (_, __) => const SizedBox(),
          ),
        ),
      );
      await tester.pump();

      expect(external.initCalls, 0);
    },
  );

  testWidgets('owned scope disposes and value scope does not dispose', (
    tester,
  ) async {
    late CounterViewModel owned;
    await tester.pumpWidget(
      Directionality(
        textDirection: TextDirection.ltr,
        child: ViewModelOwner<CounterViewModel, CounterArgs>(
          args: (_) => const CounterArgs(),
          create: (_, args) => owned = CounterViewModel(args),
          builder: (_, __) => const SizedBox(),
        ),
      ),
    );
    await tester.pumpWidget(const SizedBox());
    expect(owned.disposed, isTrue);

    final external = CounterViewModel(const CounterArgs());
    await tester.pumpWidget(
      Directionality(
        textDirection: TextDirection.ltr,
        child: ViewModelOwner<CounterViewModel, CounterArgs>.value(
          value: external,
          builder: (_, __) => const SizedBox(),
        ),
      ),
    );
    await tester.pumpWidget(const SizedBox());
    expect(external.disposed, isFalse);
  });

  testWidgets(
    'value scope can swap external view model without disposing either',
    (tester) async {
      final first = CounterViewModel(const CounterArgs());
      final second = CounterViewModel(const CounterArgs());

      await tester.pumpWidget(
        Directionality(
          textDirection: TextDirection.ltr,
          child: ViewModelOwner<CounterViewModel, CounterArgs>.value(
            value: first,
            builder: (_, viewModel) => Text('${viewModel.state}'),
          ),
        ),
      );
      expect(find.text('0'), findsOneWidget);

      second.setCount(2);
      await tester.pumpWidget(
        Directionality(
          textDirection: TextDirection.ltr,
          child: ViewModelOwner<CounterViewModel, CounterArgs>.value(
            value: second,
            builder: (_, viewModel) => Text('${viewModel.state}'),
          ),
        ),
      );

      expect(find.text('2'), findsOneWidget);
      expect(first.disposed, isFalse);
      expect(second.disposed, isFalse);
    },
  );

  testWidgets('owned scope reports dependency injection failures clearly', (
    tester,
  ) async {
    await tester.pumpWidget(
      Directionality(
        textDirection: TextDirection.ltr,
        child: ViewModelOwner<CounterViewModel, CounterArgs>(
          debugName: 'CounterViewModelScope',
          args: (_) => throw StateError('missing repository'),
          create: (_, args) => CounterViewModel(args),
          builder: (_, __) => const SizedBox(),
        ),
      ),
    );

    final error = tester.takeException();

    expect(error, isA<StateError>());
    expect(error.toString(), contains('CounterViewModelScope'));
    expect(error.toString(), contains('missing repository'));
  });

  testWidgets('owned scope reports missing generated factories clearly', (
    tester,
  ) async {
    await tester.pumpWidget(
      Directionality(
        textDirection: TextDirection.ltr,
        child: ViewModelOwner<CounterViewModel, CounterArgs>(
          debugName: 'BrokenScope',
          args: null,
          create: (_, args) => CounterViewModel(args),
          builder: (_, __) => const SizedBox(),
        ),
      ),
    );

    final error = tester.takeException();

    expect(error, isA<StateError>());
    expect(
      error.toString(),
      contains('Owned ViewModelOwner requires args and create'),
    );
  });
}

final class CounterStateForAnnotation {}

final class CounterArgs extends ViewModelArgs {
  const CounterArgs({super.observer});
}

final class CounterViewModel extends ViewModelBase<int, CounterArgs> {
  CounterViewModel(super.args) : super(initialState: 0);

  int initCalls = 0;
  bool disposed = false;
  bool failNextInit = false;

  @override
  Future<void> onInit() async {
    initCalls += 1;
    if (failNextInit) {
      failNextInit = false;
      throw StateError('init failed');
    }
  }

  Future<void> loadCount(Completer<int> count) async {
    final token = beginAction(#loadCount);
    final next = await count.future;
    if (!isCurrentAction(token)) return;
    emit(next);
  }

  void cancelLoadCount() {
    cancelAction(#loadCount);
  }

  void setCount(int count) => emit(count);

  void showToast(String message) => emitEffect(message);

  @override
  void dispose() {
    disposed = true;
    super.dispose();
  }
}

final class BareViewModel extends ViewModelBase<int, CounterArgs> {
  BareViewModel(super.args) : super(initialState: 0);
}

final class RecordingObserver implements StateObserver {
  final changes = <String>[];
  final effects = <Object>[];

  @override
  void onChanged(Object viewModel, Object? previous, Object? next) {
    changes.add('$previous -> $next');
  }

  @override
  void onEffect(Object viewModel, Object effect) {
    effects.add(effect);
  }
}
