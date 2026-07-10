import 'package:dust_flutter/route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('router diagnostics are disabled by default', () async {
    final messages = <String>[];
    final previousDebugPrint = debugPrint;
    debugPrint = (message, {wrapWidth}) {
      if (message != null) messages.add(message);
    };

    try {
      final delegate = GeneratedRouterDelegate<_TestRoute>(_runtimeConfig());
      await delegate.debugWaitForScheduledRefresh();
      await delegate.setNewRoutePath(const _TestRoute('/private'));

      expect(messages, isEmpty);
    } finally {
      debugPrint = previousDebugPrint;
    }
  });

  test('router diagnostics log redirects guards and commits', () async {
    final messages = <String>[];
    final previousDebugPrint = debugPrint;
    debugPrint = (message, {wrapWidth}) {
      if (message != null) messages.add(message);
    };

    try {
      final delegate = GeneratedRouterDelegate<_TestRoute>(
        _runtimeConfig(
          router: _DebugRouter(),
          resolveGuards: (route) {
            if (route.location != '/guard-private') return const [];
            return [const _LoginGuard()];
          },
        ),
      );
      await delegate.debugWaitForScheduledRefresh();
      await delegate.setNewRoutePath(const _TestRoute('/private'));
      await delegate.setNewRoutePath(const _TestRoute('/guard-private'));

      const fullPathsLog = 'AppRouter: Full paths for routes:\n'
          '           => /safe\n'
          '           => /private\n'
          '           => /guard-private\n'
          '           => /login\n'
          '           => /nested/:id';
      const namedPathsLog = 'AppRouter: known full paths for route names:\n'
          '           safe => /safe\n'
          '           private => /private\n'
          '           guardPrivate => /guard-private\n'
          '           login => /login\n'
          '           nestedDetail => /nested/:id';

      expect(
        messages,
        containsAllInOrder([
          fullPathsLog,
          namedPathsLog,
          'AppRouter: setting initial route /safe',
          'AppRouter: refreshing /safe',
          'AppRouter: replace /safe',
          'AppRouter: stack [/safe]',
          'AppRouter: restoring /private',
          'AppRouter: redirecting /private => /login',
          'AppRouter: stack [/login]',
          'AppRouter: restoring /guard-private',
          'AppRouter: guards 1 for /guard-private',
          'AppRouter: guard redirect /guard-private => /login',
          'AppRouter: restoring /login',
          'AppRouter: stack [/login]',
        ]),
      );
    } finally {
      debugPrint = previousDebugPrint;
    }
  });

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

  test('push future completes when delegate pops the route', () async {
    final delegate = GeneratedRouterDelegate<_TestRoute>(_runtimeConfig());
    await delegate.debugWaitForScheduledRefresh();

    final result = delegate.push<void>(const _TestRoute('/detail'));

    expect(delegate.stack.map((route) => route.location), [
      '/safe',
      '/detail',
    ]);

    await delegate.popRoute();

    await expectLater(result, completion(isNull));
  });

  testWidgets('push future completes with the Navigator pop result', (
    tester,
  ) async {
    final delegate = GeneratedRouterDelegate<_TestRoute>(
      _runtimeConfig(
        buildChild: (context, route) {
          if (route.location != '/detail') return Text(route.location);
          return TextButton(
            onPressed: () => Navigator.of(context).pop('saved'),
            child: const Text('close detail'),
          );
        },
      ),
    );

    await tester.pumpWidget(
      MaterialApp.router(
        routerConfig: RouterConfig<_TestRoute>(
          routeInformationProvider: PlatformRouteInformationProvider(
            initialRouteInformation: RouteInformation(
              uri: Uri.parse('/safe'),
            ),
          ),
          routeInformationParser: GeneratedRouteInformationParser<_TestRoute>(
            parseRoute: (uri) => _TestRoute(uri.toString()),
            routeLocation: (route) => route.location,
          ),
          routerDelegate: delegate,
          backButtonDispatcher: RootBackButtonDispatcher(),
        ),
      ),
    );
    await tester.pumpAndSettle();

    final result = delegate.push<String>(const _TestRoute('/detail'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('close detail'));
    await tester.pumpAndSettle();

    await expectLater(result, completion('saved'));
  });
}

RouterRuntimeConfig<_TestRoute> _runtimeConfig({
  RouterBase<_TestRoute>? router,
  RouteGuardResolver<_TestRoute>? resolveGuards,
  Widget Function(BuildContext context, _TestRoute route)? buildChild,
}) {
  return RouterRuntimeConfig<_TestRoute>(
    router: router ?? _NoRedirectRouter(),
    initialRoute: const _TestRoute('/safe'),
    parseRoute: (uri) => _TestRoute(uri.toString()),
    routeLocation: (route) => route.location,
    requiresAuth: (_) => false,
    resolveGuards: resolveGuards ?? (_) => const [],
    buildPage: (route, key, onPopInvoked) => MaterialPage<Object?>(
      key: key,
      name: route.location,
      onPopInvoked: onPopInvoked,
      child: Builder(
        builder: (context) =>
            buildChild?.call(context, route) ?? const SizedBox(),
      ),
    ),
    debugRoutes: const [
      GeneratedRoute('/safe', page: SizedBox, name: 'safe'),
      GeneratedRoute('/private', page: SizedBox, name: 'private'),
      GeneratedRoute('/guard-private', page: SizedBox, name: 'guardPrivate'),
      GeneratedRoute('/login', page: SizedBox, name: 'login'),
      GeneratedRoute(
        '/nested',
        routes: [
          GeneratedRoute(':id', page: SizedBox, name: 'nestedDetail'),
        ],
      ),
    ],
  );
}

final class _TestRoute {
  const _TestRoute(this.location);

  final String location;
}

final class _NoRedirectRouter extends RouterBase<_TestRoute> {}

final class _DebugRouter extends RouterBase<_TestRoute> {
  @override
  bool get debugLogDiagnostics => true;

  @override
  _TestRoute? redirect(_TestRoute route) {
    return route.location == '/private' ? const _TestRoute('/login') : null;
  }
}

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

final class _LoginGuard implements RouteGuard<_TestRoute> {
  const _LoginGuard();

  @override
  _TestRoute? canActivate(_TestRoute route) => const _TestRoute('/login');
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
