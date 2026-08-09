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
}

RouterRuntimeConfig<_TestRoute> _runtimeConfig({
  RouterBase<_TestRoute>? router,
}) {
  return RouterRuntimeConfig<_TestRoute>(
    router: router ?? _NoRedirectRouter(),
    initialRoute: const _TestRoute('/safe'),
    parseRoute: (uri) => _TestRoute(uri.toString()),
    routeLocation: (route) => route.location,
    requiresAuth: (_) => false,
    resolveGuards: (_) => const [],
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
