import 'package:dust_flutter/state.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('renders child when scopes are empty', (tester) async {
    await tester.pumpWidget(
      const ViewModelScopes(
        child: Text('child', textDirection: TextDirection.ltr),
      ),
    );

    expect(find.text('child'), findsOneWidget);
  });

  testWidgets('nests scopes in list order', (tester) async {
    await tester.pumpWidget(
      ViewModelScopes(
        scopes: [
          (child) => _TestScope(name: 'first', child: child),
          (child) => _TestScope(name: 'second', child: child),
        ],
        child: const Text('child', textDirection: TextDirection.ltr),
      ),
    );

    final first = find.byKey(const ValueKey<String>('first'));
    final second = find.byKey(const ValueKey<String>('second'));
    final child = find.text('child');

    expect(find.descendant(of: first, matching: second), findsOneWidget);
    expect(find.descendant(of: second, matching: child), findsOneWidget);
  });

  testWidgets('updates when scope list changes', (tester) async {
    Widget build(List<String> names) {
      return ViewModelScopes(
        scopes: [
          for (final name in names)
            (child) => _TestScope(name: name, child: child),
        ],
        child: const Text('child', textDirection: TextDirection.ltr),
      );
    }

    await tester.pumpWidget(build(['first']));

    expect(find.byKey(const ValueKey<String>('first')), findsOneWidget);
    expect(find.byKey(const ValueKey<String>('second')), findsNothing);

    await tester.pumpWidget(build(['first', 'second']));

    final first = find.byKey(const ValueKey<String>('first'));
    final second = find.byKey(const ValueKey<String>('second'));
    expect(find.descendant(of: first, matching: second), findsOneWidget);

    await tester.pumpWidget(build([]));

    expect(find.byKey(const ValueKey<String>('first')), findsNothing);
    expect(find.byKey(const ValueKey<String>('second')), findsNothing);
    expect(find.text('child'), findsOneWidget);
  });
}

final class _TestScope extends StatelessWidget {
  const _TestScope({required this.name, required this.child});

  final String name;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return KeyedSubtree(
      key: ValueKey<String>(name),
      child: child,
    );
  }
}
