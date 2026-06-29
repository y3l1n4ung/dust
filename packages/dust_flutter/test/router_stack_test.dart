import 'package:dust_flutter/route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('duplicate pushes get distinct page keys', () async {
    final delegate = GeneratedRouterDelegate<_TestRoute>(
      _runtimeConfig(),
    );
    await _drainInitialRefresh();

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
    await _drainInitialRefresh();

    final initialKey = delegate.debugPageKeys.single;

    delegate.replace(const _TestRoute('/safe'));
    await _drainInitialRefresh();

    expect(delegate.debugPageKeys.single, same(initialKey));
  });
}

Future<void> _drainInitialRefresh() async {
  await Future<void>.delayed(Duration.zero);
}

RouterRuntimeConfig<_TestRoute> _runtimeConfig() {
  return RouterRuntimeConfig<_TestRoute>(
    router: _NoRedirectRouter(),
    initialRoute: const _TestRoute('/safe'),
    parseRoute: (uri) => _TestRoute(uri.toString()),
    routeLocation: (route) => route.location,
    requiresAuth: (_) => false,
    resolveGuards: (_) => const [],
    buildPage: (route, key) {
      return MaterialPage<void>(
        key: key,
        name: route.location,
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
