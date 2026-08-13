import 'package:dust_flutter/route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('duplicate pushes get distinct page keys', () async {
    final delegate = GeneratedRouterDelegate<_TestRoute>(
      _runtimeConfig(),
    );
    await delegate.debugWaitForScheduledRefresh();

    delegate
      ..push(const _TestRoute('/same'))
      ..push(const _TestRoute('/same'));

    expect(delegate.stack.map((route) => route.location), [
      '/safe',
      '/same',
      '/same',
    ]);
    expect(delegate.debugPageKeys.length, 3);
    expect(delegate.debugPageKeys[1], isNot(delegate.debugPageKeys[2]));
  });

  test('same-location replace preserves the page key', () async {
    final delegate = GeneratedRouterDelegate<_TestRoute>(
      _runtimeConfig(),
    );
    await delegate.debugWaitForScheduledRefresh();

    final initialKey = delegate.debugPageKeys.single;

    delegate.replace(const _TestRoute('/safe'));
    await delegate.debugWaitForScheduledRefresh();

    expect(delegate.debugPageKeys.single, same(initialKey));
  });

  test('replace with a new location completes the replaced push future',
      () async {
    final delegate = GeneratedRouterDelegate<_TestRoute>(
      _runtimeConfig(),
    );
    await delegate.debugWaitForScheduledRefresh();

    final result = delegate.push<void>(const _TestRoute('/detail'));
    await Future<void>.delayed(Duration.zero);

    delegate.replace(const _TestRoute('/other'));
    await delegate.debugWaitForScheduledRefresh();

    expect(delegate.stack.map((route) => route.location), ['/safe', '/other']);
    await expectLater(result, completion(isNull));
  });

  test('go completes all pending push futures with null', () async {
    final delegate = GeneratedRouterDelegate<_TestRoute>(
      _runtimeConfig(),
    );
    await delegate.debugWaitForScheduledRefresh();

    final first = delegate.push<void>(const _TestRoute('/first'));
    final second = delegate.push<void>(const _TestRoute('/second'));
    await Future<void>.delayed(Duration.zero);

    delegate.go(const _TestRoute('/safe'));
    await delegate.debugWaitForScheduledRefresh();

    expect(delegate.stack.map((route) => route.location), ['/safe']);
    await expectLater(first, completion(isNull));
    await expectLater(second, completion(isNull));
  });

  test('root pop is ignored without reporting a stack change', () async {
    final router = _RecordingRouter();
    final delegate = GeneratedRouterDelegate<_TestRoute>(
      _runtimeConfig(router: router),
    );
    await delegate.debugWaitForScheduledRefresh();

    expect(delegate.pop(), isFalse);

    expect(delegate.stack.map((route) => route.location), ['/safe']);
    expect(router.stackChanges, isEmpty);
  });

  test('switching branches preserves each branch stack', () async {
    final delegate = GeneratedRouterDelegate<_TestRoute>(
      _runtimeConfig(
        routeBranch: (route) => switch (route.location) {
          '/home' || '/home/detail' => 'mainTabs',
          '/orders' || '/orders/detail' => 'ordersTabs',
          _ => null,
        },
      ),
    );
    await delegate.debugWaitForScheduledRefresh();

    delegate.go(const _TestRoute('/home'));
    await Future<void>.delayed(Duration.zero);
    final homeDetail = delegate.push<void>(const _TestRoute('/home/detail'));
    await Future<void>.delayed(Duration.zero);

    delegate.go(const _TestRoute('/orders'));
    await Future<void>.delayed(Duration.zero);
    final orderDetail = delegate.push<void>(const _TestRoute('/orders/detail'));
    await Future<void>.delayed(Duration.zero);

    expect(delegate.stack.map((route) => route.location), [
      '/orders',
      '/orders/detail',
    ]);

    delegate.go(const _TestRoute('/home'));
    await Future<void>.delayed(Duration.zero);

    expect(delegate.stack.map((route) => route.location), [
      '/home',
      '/home/detail',
    ]);

    delegate.go(const _TestRoute('/orders'));
    await Future<void>.delayed(Duration.zero);

    expect(delegate.stack.map((route) => route.location), [
      '/orders',
      '/orders/detail',
    ]);

    delegate.dispose();
    await expectLater(homeDetail, completion(isNull));
    await expectLater(orderDetail, completion(isNull));
  });

  test('non-branch routes keep single stack behavior', () async {
    final delegate = GeneratedRouterDelegate<_TestRoute>(
      _runtimeConfig(routeBranch: (_) => null),
    );
    await delegate.debugWaitForScheduledRefresh();

    delegate
      ..push(const _TestRoute('/first'))
      ..push(const _TestRoute('/second'));
    await Future<void>.delayed(Duration.zero);

    delegate.go(const _TestRoute('/safe'));
    await Future<void>.delayed(Duration.zero);

    expect(delegate.stack.map((route) => route.location), ['/safe']);
  });

  test('deep-link restoration activates branch without losing hidden stacks',
      () async {
    final delegate = GeneratedRouterDelegate<_TestRoute>(
      _runtimeConfig(
        routeBranch: _testBranch,
        restoreStack: (route) => switch (route.location) {
          '/home/detail' => [
              const _TestRoute('/home'),
              route,
            ],
          '/orders/detail' => [
              const _TestRoute('/orders'),
              route,
            ],
          _ => [route],
        },
      ),
    );
    await delegate.debugWaitForScheduledRefresh();

    await delegate.setNewRoutePath(const _TestRoute('/home/detail'));
    expect(delegate.stack.map((route) => route.location), [
      '/home',
      '/home/detail',
    ]);

    await delegate.setNewRoutePath(const _TestRoute('/orders/detail'));
    expect(delegate.stack.map((route) => route.location), [
      '/orders',
      '/orders/detail',
    ]);

    delegate.go(const _TestRoute('/home'));
    await Future<void>.delayed(Duration.zero);
    expect(delegate.stack.map((route) => route.location), [
      '/home',
      '/home/detail',
    ]);
  });

  test('browser back and forward can swap active branch stacks', () async {
    final delegate = GeneratedRouterDelegate<_TestRoute>(
      _runtimeConfig(
        routeBranch: _testBranch,
        restoreStack: (route) => route.location.endsWith('/detail')
            ? [
                _TestRoute(route.location.replaceAll('/detail', '')),
                route,
              ]
            : [route],
      ),
    );
    await delegate.debugWaitForScheduledRefresh();

    await delegate.setNewRoutePath(const _TestRoute('/home/detail'));
    await delegate.setNewRoutePath(const _TestRoute('/orders/detail'));

    await delegate.setNewRoutePath(const _TestRoute('/home/detail'));
    expect(delegate.stack.map((route) => route.location), [
      '/home',
      '/home/detail',
    ]);

    await delegate.setNewRoutePath(const _TestRoute('/orders/detail'));
    expect(delegate.stack.map((route) => route.location), [
      '/orders',
      '/orders/detail',
    ]);
  });
}

String? _testBranch(_TestRoute route) => switch (route.location) {
      '/home' || '/home/detail' => 'mainTabs',
      '/orders' || '/orders/detail' => 'ordersTabs',
      _ => null,
    };

RouterRuntimeConfig<_TestRoute> _runtimeConfig({
  RouterBase<_TestRoute>? router,
  RouteBranch<_TestRoute>? routeBranch,
  RouteStackRestorer<_TestRoute>? restoreStack,
}) {
  return RouterRuntimeConfig<_TestRoute>(
    router: router ?? _NoRedirectRouter(),
    initialRoute: const _TestRoute('/safe'),
    parseRoute: (uri) => _TestRoute(uri.toString()),
    routeLocation: (route) => route.location,
    requiresAuth: (_) => false,
    routeBranch: routeBranch ?? (_) => null,
    resolveGuards: (_) => const [],
    restoreStack: restoreStack,
    buildPage: (route, key, onPopInvoked) {
      return MaterialPage<Object?>(
        key: key,
        name: route.location,
        onPopInvoked: onPopInvoked,
        child: const SizedBox(),
      );
    },
  );
}

final class _TestRoute {
  const _TestRoute(this.location);

  final String location;
}

final class _NoRedirectRouter extends RouterBase<_TestRoute> {}

final class _RecordingRouter extends RouterBase<_TestRoute> {
  final stackChanges = <String>[];

  @override
  void didChangeRouteStack(
    RouteStack<_TestRoute> previous,
    RouteStack<_TestRoute> next,
  ) {
    stackChanges.add('${_locations(previous)} => ${_locations(next)}');
  }

  String _locations(RouteStack<_TestRoute> stack) {
    return '[${stack.map((route) => route.location).join(', ')}]';
  }
}
