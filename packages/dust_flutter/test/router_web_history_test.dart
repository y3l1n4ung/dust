import 'package:dust_flutter/route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('browser back and forward restore typed route stacks', () async {
    final delegate = GeneratedRouterDelegate<_Route>(
      _runtimeConfig(restoreStack: _restoreStack),
    );
    await delegate.debugWaitForScheduledRefresh();

    await delegate.setNewRoutePath(const _Route('/product/7'));
    expect(delegate.stack.map((route) => route.location), ['/', '/product/7']);

    await delegate.setNewRoutePath(const _Route('/orders/ORDER-1'));
    expect(delegate.stack.map((route) => route.location), [
      '/',
      '/orders',
      '/orders/ORDER-1',
    ]);

    await delegate.setNewRoutePath(const _Route('/product/7'));
    expect(delegate.stack.map((route) => route.location), ['/', '/product/7']);
  });

  test('query-only browser navigation restores a distinct location', () async {
    final router = _RecordingRouter();
    final delegate = GeneratedRouterDelegate<_Route>(
      _runtimeConfig(router: router, restoreStack: _restoreStack),
    );
    await delegate.debugWaitForScheduledRefresh();

    await delegate.setNewRoutePath(const _Route('/search?query=bag&page=1'));
    await delegate
        .setNewRoutePath(const _Route('/search?query=bag&page=2#grid'));

    expect(delegate.currentConfiguration.location,
        '/search?query=bag&page=2#grid');
    expect(delegate.stack.map((route) => route.location), [
      '/',
      '/search?query=bag&page=2#grid',
    ]);
    expect(router.stackChanges, [
      '[/] => [/, /search?query=bag&page=1]',
      '[/, /search?query=bag&page=1] => [/, /search?query=bag&page=2#grid]',
    ]);
  });

  test('shell-wrapped deep links report the restored route shell', () async {
    final messages = <String>[];
    final previousDebugPrint = debugPrint;
    debugPrint = (message, {wrapWidth}) {
      if (message != null) messages.add(message);
    };

    try {
      final delegate = GeneratedRouterDelegate<_Route>(
        _runtimeConfig(
          router: _DebugRouter(),
          restoreStack: _restoreStack,
        ),
      );
      await delegate.debugWaitForScheduledRefresh();
      messages.clear();

      await delegate.setNewRoutePath(const _Route('/app/settings'));

      expect(delegate.stack.map((route) => route.location), [
        '/',
        '/app',
        '/app/settings',
      ]);
      expect(messages, [
        'AppRouter: restoring /app/settings',
        'AppRouter: route /app/settings name=settings shell=AppShell branch=-',
        'AppRouter: stack [/, /app, /app/settings]',
      ]);
    } finally {
      debugPrint = previousDebugPrint;
    }
  });
}

RouteStack<_Route> _restoreStack(_Route route) {
  return switch (route.location) {
    '/product/7' => [const _Route('/'), route],
    '/orders/ORDER-1' => [
        const _Route('/'),
        const _Route('/orders'),
        route,
      ],
    '/search?query=bag&page=1' || '/search?query=bag&page=2#grid' => [
        const _Route('/'),
        route
      ],
    '/app/settings' => [
        const _Route('/'),
        const _Route('/app'),
        route,
      ],
    _ => [route],
  };
}

RouterRuntimeConfig<_Route> _runtimeConfig({
  RouterBase<_Route>? router,
  RouteStackRestorer<_Route>? restoreStack,
}) {
  return RouterRuntimeConfig<_Route>(
    router: router ?? _NoRedirectRouter(),
    initialRoute: const _Route('/'),
    parseRoute: (uri) => _Route(uri.toString()),
    routeLocation: (route) => route.location,
    requiresAuth: (_) => false,
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
    debugRoutes: const [
      GeneratedRoute('/', page: SizedBox, name: 'home'),
      GeneratedRoute('/app', page: SizedBox, name: 'app', shell: AppShell),
      GeneratedRoute(
        '/app',
        routes: [
          GeneratedRoute('settings', page: SizedBox, name: 'settings'),
        ],
      ),
    ],
    debugInfo: (route) => switch (route.location) {
      '/' => const RouteDebugInfo(name: 'home'),
      '/app' => const RouteDebugInfo(name: 'app', shell: 'AppShell'),
      '/app/settings' => const RouteDebugInfo(
          name: 'settings',
          shell: 'AppShell',
        ),
      _ => const RouteDebugInfo(),
    },
  );
}

final class _Route {
  const _Route(this.location);

  final String location;
}

final class _NoRedirectRouter extends RouterBase<_Route> {}

final class _DebugRouter extends RouterBase<_Route> {
  @override
  bool get debugLogDiagnostics => true;
}

final class _RecordingRouter extends RouterBase<_Route> {
  final stackChanges = <String>[];

  @override
  void didChangeRouteStack(
      RouteStack<_Route> previous, RouteStack<_Route> next) {
    stackChanges.add('${_locations(previous)} => ${_locations(next)}');
  }

  String _locations(RouteStack<_Route> stack) {
    return '[${stack.map((route) => route.location).join(', ')}]';
  }
}

final class AppShell extends StatelessWidget {
  const AppShell({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) => child;
}
