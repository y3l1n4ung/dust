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

  test('init runs once', () async {
    final vm = CounterViewModel(const CounterArgs());

    await Future.wait([vm.init(), vm.init()]);
    await vm.init();

    expect(vm.initCalls, 1);
  });

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
}

final class CounterArgs extends ViewModelArgs {
  const CounterArgs({super.observer});
}

final class CounterViewModel extends ViewModelBase<int, CounterArgs> {
  CounterViewModel(super.args) : super(initialState: 0);

  int initCalls = 0;
  bool disposed = false;

  @override
  Future<void> onInit() async {
    initCalls += 1;
  }

  void setCount(int count) => emit(count);

  void showToast(String message) => emitEffect(message);

  @override
  void dispose() {
    disposed = true;
    super.dispose();
  }
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
