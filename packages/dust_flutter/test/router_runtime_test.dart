import 'package:dust_flutter/route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('router redirects throw StateError after the redirect cap', () async {
    final delegate = GeneratedRouterDelegate<_TestRoute>(
      _runtimeConfig(
        router: _RouterRedirectCycle(),
      ),
    );
    await delegate.debugWaitForScheduledRefresh();

    await expectLater(
      delegate.setNewRoutePath(const _TestRoute('/one')),
      throwsA(
        isA<StateError>().having(
          (error) => error.message,
          'message',
          contains('redirect cap'),
        ),
      ),
    );
  });

  test('guard redirects throw StateError after the redirect cap', () async {
    final guard = _GuardRedirectCycle();
    final delegate = GeneratedRouterDelegate<_TestRoute>(
      _runtimeConfig(
        resolveGuards: (route) {
          if (!route.location.startsWith('/guard')) return const [];
          return [guard];
        },
      ),
    );
    await delegate.debugWaitForScheduledRefresh();

    await expectLater(
      delegate.setNewRoutePath(const _TestRoute('/guard-one')),
      throwsA(
        isA<StateError>().having(
          (error) => error.message,
          'message',
          contains('guard redirect cycle'),
        ),
      ),
    );
  });
}

RouterRuntimeConfig<_TestRoute> _runtimeConfig({
  RouterBase<_TestRoute>? router,
  RouteGuardResolver<_TestRoute>? resolveGuards,
}) {
  return RouterRuntimeConfig<_TestRoute>(
    router: router ?? _NoRedirectRouter(),
    initialRoute: const _TestRoute('/safe'),
    parseRoute: (uri) => _TestRoute(uri.toString()),
    routeLocation: (route) => route.location,
    requiresAuth: (_) => false,
    resolveGuards: resolveGuards ?? (_) => const [],
    buildPage: (route, key) => MaterialPage<void>(
      key: key,
      child: const SizedBox(),
    ),
  );
}

final class _TestRoute {
  const _TestRoute(this.location);

  final String location;
}

final class _NoRedirectRouter extends RouterBase<_TestRoute> {}

final class _RouterRedirectCycle extends RouterBase<_TestRoute> {
  @override
  _TestRoute? redirect(_TestRoute route) {
    return switch (route.location) {
      '/one' => const _TestRoute('/two'),
      '/two' => const _TestRoute('/one'),
      _ => null,
    };
  }
}

final class _GuardRedirectCycle implements RouteGuard<_TestRoute> {
  @override
  _TestRoute? canActivate(_TestRoute route) {
    return switch (route.location) {
      '/guard-one' => const _TestRoute('/guard-two'),
      '/guard-two' => const _TestRoute('/guard-one'),
      _ => null,
    };
  }
}
