import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

/// Benchmark: StatefulWidget + InheritedNotifier vs Global + InheritedNotifier

// ============================================
// Approach 1: StatefulWidget (current)
// ============================================
class CounterNotifier1 extends ChangeNotifier {
  int _value = 0;
  int get value => _value;
  void increment() {
    _value++;
    notifyListeners();
  }
}

class StatefulProvider extends StatefulWidget {
  final Widget child;
  const StatefulProvider({super.key, required this.child});

  @override
  State<StatefulProvider> createState() => _StatefulProviderState();
}

class _StatefulProviderState extends State<StatefulProvider> {
  late final CounterNotifier1 _notifier = CounterNotifier1();

  @override
  void dispose() {
    _notifier.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return _StatefulInherited(notifier: _notifier, child: widget.child);
  }
}

class _StatefulInherited extends InheritedNotifier<CounterNotifier1> {
  const _StatefulInherited({required super.notifier, required super.child});
}

extension StatefulContext on BuildContext {
  CounterNotifier1 get counter1 =>
      dependOnInheritedWidgetOfExactType<_StatefulInherited>()!.notifier!;
  CounterNotifier1 readCounter1() =>
      getInheritedWidgetOfExactType<_StatefulInherited>()!.notifier!;
}

// ============================================
// Approach 2: Global instance (simpler)
// ============================================
class CounterNotifier2 extends ChangeNotifier {
  int _value = 0;
  int get value => _value;
  void increment() {
    _value++;
    notifyListeners();
  }

  void reset() {
    _value = 0;
  }
}

final _globalCounter = CounterNotifier2();

class GlobalProvider extends InheritedNotifier<CounterNotifier2> {
  GlobalProvider({super.key, required super.child})
      : super(notifier: _globalCounter);
}

extension GlobalContext on BuildContext {
  CounterNotifier2 get counter2 =>
      dependOnInheritedWidgetOfExactType<GlobalProvider>()!.notifier!;
  CounterNotifier2 readCounter2() =>
      getInheritedWidgetOfExactType<GlobalProvider>()!.notifier!;
}

// ============================================
// Test wrapper
// ============================================
Widget wrapWithApp(Widget child) {
  return MaterialApp(home: Scaffold(body: child));
}

// ============================================
// Benchmark Tests
// ============================================
void main() {
  group('Provider Performance Benchmark', () {
    testWidgets('Approach 1: StatefulWidget - initial build', (tester) async {
      final stopwatch = Stopwatch()..start();

      for (int i = 0; i < 100; i++) {
        await tester.pumpWidget(
          wrapWithApp(
            StatefulProvider(
              child: Builder(
                builder: (context) => Text('${context.counter1.value}'),
              ),
            ),
          ),
        );
      }

      stopwatch.stop();
      debugPrint(
          'StatefulWidget x100 builds: ${stopwatch.elapsedMilliseconds}ms');
    });

    testWidgets('Approach 2: Global - initial build', (tester) async {
      _globalCounter.reset();
      final stopwatch = Stopwatch()..start();

      for (int i = 0; i < 100; i++) {
        await tester.pumpWidget(
          wrapWithApp(
            GlobalProvider(
              child: Builder(
                builder: (context) => Text('${context.counter2.value}'),
              ),
            ),
          ),
        );
      }

      stopwatch.stop();
      debugPrint(
          'GlobalProvider x100 builds: ${stopwatch.elapsedMilliseconds}ms');
    });

    testWidgets('Approach 1: StatefulWidget - state updates', (tester) async {
      late CounterNotifier1 notifier;

      await tester.pumpWidget(
        wrapWithApp(
          StatefulProvider(
            child: Builder(
              builder: (context) {
                notifier = context.counter1;
                return Text('${notifier.value}');
              },
            ),
          ),
        ),
      );

      final stopwatch = Stopwatch()..start();

      for (int i = 0; i < 1000; i++) {
        notifier.increment();
        await tester.pump();
      }

      stopwatch.stop();
      debugPrint(
          'StatefulWidget x1000 updates: ${stopwatch.elapsedMilliseconds}ms');
    });

    testWidgets('Approach 2: Global - state updates', (tester) async {
      _globalCounter.reset();

      await tester.pumpWidget(
        wrapWithApp(
          GlobalProvider(
            child: Builder(
              builder: (context) {
                final counter = context.counter2;
                return Text('${counter.value}');
              },
            ),
          ),
        ),
      );

      final stopwatch = Stopwatch()..start();

      for (int i = 0; i < 1000; i++) {
        _globalCounter.increment();
        await tester.pump();
      }

      stopwatch.stop();
      debugPrint(
          'GlobalProvider x1000 updates: ${stopwatch.elapsedMilliseconds}ms');
    });

    testWidgets('Approach 1: Deep tree - only consumer rebuilds',
        (tester) async {
      late CounterNotifier1 notifier;
      int consumerRebuildCount = 0;
      int staticRebuildCount = 0;

      await tester.pumpWidget(
        wrapWithApp(
          StatefulProvider(
            child: Column(
              children: [
                // Non-subscribing widget
                Builder(builder: (_) {
                  staticRebuildCount++;
                  return const Text('Static');
                }),
                // Subscribing widget
                Builder(
                  builder: (context) {
                    notifier = context.counter1;
                    consumerRebuildCount++;
                    return Text('Counter: ${notifier.value}');
                  },
                ),
              ],
            ),
          ),
        ),
      );

      // Reset after initial build
      consumerRebuildCount = 0;
      staticRebuildCount = 0;

      for (int i = 0; i < 100; i++) {
        notifier.increment();
        await tester.pump();
      }

      debugPrint(
          'StatefulWidget: Consumer=$consumerRebuildCount, Static=$staticRebuildCount');
      expect(consumerRebuildCount, 100,
          reason: 'Consumer should rebuild 100 times');
      expect(staticRebuildCount, 0, reason: 'Static should NOT rebuild');
    });

    testWidgets('Approach 2: Deep tree - only consumer rebuilds',
        (tester) async {
      _globalCounter.reset();
      int consumerRebuildCount = 0;
      int staticRebuildCount = 0;

      await tester.pumpWidget(
        wrapWithApp(
          GlobalProvider(
            child: Column(
              children: [
                // Non-subscribing widget
                Builder(builder: (_) {
                  staticRebuildCount++;
                  return const Text('Static');
                }),
                // Subscribing widget
                Builder(
                  builder: (context) {
                    context.counter2; // subscribe
                    consumerRebuildCount++;
                    return Text('Counter: ${_globalCounter.value}');
                  },
                ),
              ],
            ),
          ),
        ),
      );

      // Reset after initial build
      consumerRebuildCount = 0;
      staticRebuildCount = 0;

      for (int i = 0; i < 100; i++) {
        _globalCounter.increment();
        await tester.pump();
      }

      debugPrint(
          'GlobalProvider: Consumer=$consumerRebuildCount, Static=$staticRebuildCount');
      expect(consumerRebuildCount, 100,
          reason: 'Consumer should rebuild 100 times');
      expect(staticRebuildCount, 0, reason: 'Static should NOT rebuild');
    });
  });

  group('Memory & Lifecycle', () {
    testWidgets('StatefulWidget disposes correctly', (tester) async {
      bool disposed = false;

      await tester.pumpWidget(
        wrapWithApp(
          _DisposableStatefulProvider(
            onDispose: () => disposed = true,
            child: const Text('test'),
          ),
        ),
      );

      expect(disposed, false);

      // Remove from tree
      await tester.pumpWidget(wrapWithApp(const SizedBox()));

      expect(disposed, true);
      debugPrint('StatefulWidget: Disposed correctly ✓');
    });

    testWidgets('Global instance never disposes', (tester) async {
      debugPrint('GlobalProvider: No dispose (by design) ✓');
      // Global lives forever - expected behavior
    });
  });
}

// Helper for dispose test
class _DisposableNotifier extends ChangeNotifier {
  final VoidCallback onDispose;
  _DisposableNotifier(this.onDispose);

  @override
  void dispose() {
    onDispose();
    super.dispose();
  }
}

class _DisposableStatefulProvider extends StatefulWidget {
  final Widget child;
  final VoidCallback onDispose;

  const _DisposableStatefulProvider({
    required this.child,
    required this.onDispose,
  });

  @override
  State<_DisposableStatefulProvider> createState() =>
      _DisposableStatefulProviderState();
}

class _DisposableStatefulProviderState
    extends State<_DisposableStatefulProvider> {
  late final _DisposableNotifier _notifier;

  @override
  void initState() {
    super.initState();
    _notifier = _DisposableNotifier(widget.onDispose);
  }

  @override
  void dispose() {
    _notifier.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
