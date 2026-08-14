import 'package:dust_flutter/route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/router_runtime_support.dart';

void main() {
  testWidgets('push future completes with the Navigator pop result', (
    tester,
  ) async {
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(
        buildChild: (context, route) {
          if (route.location != '/detail') return Text(route.location);
          return TextButton(
            onPressed: () => Navigator.of(context).pop('saved'),
            child: const Text('close detail'),
          );
        },
      ),
    );
    await pumpRuntimeRouter(tester, delegate);

    final result = delegate.push<String>(const TestRoute('/detail'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('close detail'));
    await tester.pumpAndSettle();

    await expectLater(result, completion('saved'));
  });

  testWidgets('Navigator page removal revalidates the exposed route', (
    tester,
  ) async {
    final router = AuthRouter();
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(
        router: router,
        buildChild: (context, route) {
          if (route.location != '/detail') return Text(route.location);
          return TextButton(
            onPressed: () {
              router.isAuthenticated = false;
              Navigator.of(context).pop();
            },
            child: const Text('close detail'),
          );
        },
      ),
    );
    await pumpRuntimeRouter(tester, delegate);

    final privateResult = delegate.push<void>(const TestRoute('/private'));
    await tester.pumpAndSettle();
    final detailResult = delegate.push<void>(const TestRoute('/detail'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('close detail'));
    await tester.pumpAndSettle();

    expect(delegate.stack.map((route) => route.location), [
      '/safe',
      '/login',
    ]);
    await expectLater(privateResult, completion(isNull));
    await expectLater(detailResult, completion(isNull));
  });

  testWidgets('generated router passes Navigator observers through', (
    tester,
  ) async {
    final observer = RecordingNavigatorObserver();
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(
        router: ObserverRouter([observer]),
        buildChild: (context, route) => Text(route.location),
      ),
    );
    await pumpRuntimeRouter(tester, delegate);
    observer.clear();

    final result = delegate.push<void>(const TestRoute('/detail'));
    await tester.pumpAndSettle();
    await delegate.popRoute();
    await tester.pumpAndSettle();
    await expectLater(result, completion(isNull));

    expect(observer.pushed, ['/detail']);
    expect(observer.popped, ['/detail']);
  });

  testWidgets('generated page transition runs at the route boundary', (
    tester,
  ) async {
    final transition = RecordingPageTransitionsBuilder();
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(
        transitionForRoute: (route) =>
            route.location == '/detail' ? transition : null,
        buildChild: (context, route) => Text(route.location),
      ),
    );
    await pumpRuntimeRouter(tester, delegate);

    final result = delegate.push<void>(const TestRoute('/detail'));
    await tester.pump();

    expect(transition.calls, greaterThan(0));
    expect(transition.routeNames.toSet(), {'/detail'});

    await delegate.popRoute();
    await expectLater(result, completion(isNull));
  });

  testWidgets('system back dismisses a dialog before popping the route', (
    tester,
  ) async {
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(
        buildChild: (context, route) => TextButton(
          onPressed: () => showDialog<void>(
            context: context,
            builder: (context) => const AlertDialog(content: Text('sheet')),
          ),
          child: Text(route.location),
        ),
      ),
    );
    await pumpRuntimeRouter(tester, delegate);

    delegate.push<void>(const TestRoute('/detail'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('/detail'));
    await tester.pumpAndSettle();
    expect(find.text('sheet'), findsOneWidget);

    await delegate.popRoute();
    await tester.pumpAndSettle();

    expect(
      find.text('sheet'),
      findsNothing,
      reason: 'back should dismiss the dialog first',
    );
    expect(
      delegate.stack.map((route) => route.location),
      ['/safe', '/detail'],
      reason: 'the Dust stack must be untouched while a dialog is open',
    );
  });

  testWidgets('system back dismisses a modal bottom sheet before the route', (
    tester,
  ) async {
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(
        buildChild: (context, route) => TextButton(
          onPressed: () => showModalBottomSheet<void>(
            context: context,
            builder: (context) => const Text('sheet'),
          ),
          child: Text(route.location),
        ),
      ),
    );
    await pumpRuntimeRouter(tester, delegate);

    delegate.push<void>(const TestRoute('/detail'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('/detail'));
    await tester.pumpAndSettle();
    expect(find.text('sheet'), findsOneWidget);

    await delegate.popRoute();
    await tester.pumpAndSettle();

    expect(find.text('sheet'), findsNothing);
    expect(delegate.stack.map((route) => route.location), [
      '/safe',
      '/detail',
    ]);
  });

  testWidgets('system back respects PopScope on a generated page', (
    tester,
  ) async {
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(
        buildChild: (context, route) => PopScope(
          canPop: route.location != '/detail',
          child: Text(route.location),
        ),
      ),
    );
    await pumpRuntimeRouter(tester, delegate);

    delegate.push<void>(const TestRoute('/detail'));
    await tester.pumpAndSettle();

    final popped = await delegate.popRoute();
    await tester.pumpAndSettle();

    expect(popped, isTrue, reason: 'the blocked pop was handled');
    expect(
      delegate.stack.map((route) => route.location),
      ['/safe', '/detail'],
      reason: 'PopScope(canPop: false) must keep the route on the stack',
    );
  });

  testWidgets('system back pops a generated page when nothing sits above it', (
    tester,
  ) async {
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(buildChild: (context, route) => Text(route.location)),
    );
    await pumpRuntimeRouter(tester, delegate);

    final pushed = delegate.push<String>(const TestRoute('/detail'));
    await tester.pumpAndSettle();

    final popped = await delegate.popRoute();
    await tester.pumpAndSettle();

    expect(popped, isTrue);
    expect(delegate.stack.map((route) => route.location), ['/safe']);
    await expectLater(pushed, completion(isNull));
  });

  testWidgets('system back reports unhandled on the last generated page', (
    tester,
  ) async {
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(buildChild: (context, route) => Text(route.location)),
    );
    await pumpRuntimeRouter(tester, delegate);

    final popped = await delegate.popRoute();
    await tester.pumpAndSettle();

    expect(
      popped,
      isFalse,
      reason: 'an unhandled back lets the platform close the app',
    );
    expect(delegate.stack.map((route) => route.location), ['/safe']);
  });
}
