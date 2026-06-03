import 'dart:async';

import 'package:dust_flutter/route.dart';
import 'package:flutter/material.dart' hide Route, Router;
import 'package:flutter_test/flutter_test.dart';

final class DashboardPage {
  const DashboardPage();
}

final class LoginPage {
  const LoginPage();
}

final class ProjectPage {
  const ProjectPage();
}

final class AppShell {
  const AppShell();
}

final class AuthGuard {
  const AuthGuard();
}

enum TestRoute { home, login, admin, slow, fast, guardedPublic }

void main() {
  group('Router', () {
    test('stores router root metadata', () {
      const router = Router(
        initial: DashboardPage,
        notFound: LoginPage,
        refreshListenable: 'session',
        generatedBase: r'$AppRouter',
      );

      expect(router.initial, DashboardPage);
      expect(router.notFound, LoginPage);
      expect(router.refreshListenable, 'session');
      expect(router.generatedBase, r'$AppRouter');
    });

    test('has nullable optional properties by default', () {
      const router = Router(initial: DashboardPage);

      expect(router.initial, DashboardPage);
      expect(router.notFound, isNull);
      expect(router.refreshListenable, isNull);
      expect(router.generatedBase, isNull);
    });
  });

  group('Route', () {
    test('stores page route metadata', () {
      const route = Route(
        '/projects/:projectId',
        name: 'project',
        shell: AppShell,
        guards: [AuthGuard],
        transition: CupertinoPageTransitionsBuilder(),
        fullscreenDialog: true,
        maintainState: false,
      );

      expect(route.path, '/projects/:projectId');
      expect(route.name, 'project');
      expect(route.shell, AppShell);
      expect(route.guards, [AuthGuard]);
      expect(route.transition, isA<CupertinoPageTransitionsBuilder>());
      expect(route.fullscreenDialog, isTrue);
      expect(route.maintainState, isFalse);
    });

    test('uses stable Flutter defaults', () {
      const route = Route('/login');

      expect(route.path, '/login');
      expect(route.name, isNull);
      expect(route.shell, isNull);
      expect(route.guards, isEmpty);
      expect(route.transition, isNull);
      expect(route.fullscreenDialog, isFalse);
      expect(route.maintainState, isTrue);
    });
  });

  group('GeneratedRoute', () {
    test('stores generated tree metadata', () {
      const child = GeneratedRoute(':projectId', page: ProjectPage);
      const route = GeneratedRoute(
        '/projects',
        page: ProjectPage,
        name: 'projectsShell',
        routes: [child],
        shell: AppShell,
        guards: [AuthGuard],
        transition: FadeUpwardsPageTransitionsBuilder(),
        fullscreenDialog: true,
        maintainState: false,
      );

      expect(route.path, '/projects');
      expect(route.page, ProjectPage);
      expect(route.name, 'projectsShell');
      expect(route.routes, [child]);
      expect(route.shell, AppShell);
      expect(route.guards, [AuthGuard]);
      expect(route.transition, isA<FadeUpwardsPageTransitionsBuilder>());
      expect(route.fullscreenDialog, isTrue);
      expect(route.maintainState, isFalse);
    });

    test('allows synthetic group routes without a page', () {
      const route = GeneratedRoute(
        '/projects',
        routes: [GeneratedRoute(':projectId', page: ProjectPage)],
      );

      expect(route.page, isNull);
      expect(route.routes.single.page, ProjectPage);
    });

    test('traverses generated route metadata depth-first', () {
      const route = GeneratedRoute(
        '/projects',
        routes: [
          GeneratedRoute(
            ':projectId',
            page: ProjectPage,
            name: 'project',
            routes: [
              GeneratedRoute('settings', page: LoginPage, name: 'settings'),
            ],
          ),
        ],
      );

      expect(route.depthFirst.map((route) => route.path), [
        '/projects',
        ':projectId',
        'settings',
      ]);
      expect(route.findByName('settings')?.page, LoginPage);
      expect(route.findByPage(ProjectPage)?.name, 'project');
      expect(route.findByName('missing'), isNull);
    });
  });

  group('BottomToTopPageTransitionsBuilder', () {
    test('is publicly exported for fullscreen dialog routes', () {
      const builder = BottomToTopPageTransitionsBuilder();

      expect(builder, isA<PageTransitionsBuilder>());
    });
  });

  group('DustRouterDelegate', () {
    test('supports go push replace and pop stack operations', () async {
      final delegate = DustRouterDelegate<TestRoute>(_config(_TestRouter()));

      delegate.go(TestRoute.login);
      await _flushAsync();
      expect(delegate.stack, [TestRoute.login]);

      delegate.push(TestRoute.admin);
      await _flushAsync();
      expect(delegate.stack, [TestRoute.login, TestRoute.admin]);

      delegate.replace(TestRoute.fast);
      await _flushAsync();
      expect(delegate.stack, [TestRoute.login, TestRoute.fast]);

      expect(await delegate.popRoute(), isTrue);
      expect(delegate.stack, [TestRoute.login]);
      expect(await delegate.popRoute(), isFalse);
      delegate.dispose();
    });

    test('reactive refresh reevaluates redirect', () async {
      final refresh = ChangeNotifier();
      final router = _TestRouter(refreshListenable: refresh);
      final delegate = DustRouterDelegate<TestRoute>(_config(router));
      await _flushAsync();

      router.redirectToLogin = true;
      refresh.notifyListeners();
      await _flushAsync();

      expect(delegate.currentRoute, TestRoute.login);
      delegate.dispose();
    });

    test('route guard block keeps current route', () async {
      final router = _TestRouter(guardResult: RouteGuardResult.block());
      final delegate = DustRouterDelegate<TestRoute>(_config(router));
      await _flushAsync();

      delegate.go(TestRoute.admin);
      await _flushAsync();

      expect(delegate.currentRoute, TestRoute.home);
      delegate.dispose();
    });

    test('default guard executes route guards on public routes', () async {
      final router = _TestRouter(guardResult: RouteGuardResult.block());
      final delegate = DustRouterDelegate<TestRoute>(_config(router));
      await _flushAsync();

      delegate.go(TestRoute.guardedPublic);
      await _flushAsync();

      expect(delegate.currentRoute, TestRoute.home);
      delegate.dispose();
    });

    test('stale async guard result is ignored', () async {
      final slowGuard = Completer<RouteGuardResult<TestRoute>>();
      final router = _TestRouter(
        guardForRoute: (route) => route == TestRoute.slow
            ? slowGuard.future
            : Future.value(RouteGuardResult.allow()),
      );
      final delegate = DustRouterDelegate<TestRoute>(_config(router));
      await _flushAsync();

      delegate.go(TestRoute.slow);
      delegate.go(TestRoute.fast);
      await _flushAsync();
      slowGuard.complete(RouteGuardResult.redirect(TestRoute.login));
      await _flushAsync();

      expect(delegate.currentRoute, TestRoute.fast);
      delegate.dispose();
    });

    test('browser deep link restores generated stack', () async {
      final delegate = DustRouterDelegate<TestRoute>(_config(_TestRouter()));
      await _flushAsync();

      await delegate.setNewRoutePath(TestRoute.admin);
      await _flushAsync();

      expect(delegate.stack, [TestRoute.home, TestRoute.admin]);
      delegate.dispose();
    });
  });

  group('DustRouteInformationParser', () {
    test('round-trips browser route information', () async {
      final parser = DustRouteInformationParser<TestRoute>(
        parseRoute: _config(_TestRouter()).parseRoute,
        routeLocation: _config(_TestRouter()).routeLocation,
      );

      expect(
        await parser.parseRouteInformation(
          RouteInformation(uri: Uri.parse('/admin')),
        ),
        TestRoute.admin,
      );
      expect(
        parser.restoreRouteInformation(TestRoute.guardedPublic).uri.toString(),
        '/guarded-public',
      );
    });
  });

  group('generatedPage', () {
    test('creates Material page by default', () {
      final page = generatedPage(
        location: '/',
        name: 'home',
        child: const SizedBox.shrink(),
      );

      expect(page, isA<MaterialPage<void>>());
    });

    test('injects per-route transition through Material page child', () {
      final page = generatedPage(
        location: '/checkout',
        name: 'checkout',
        transition: const BottomToTopPageTransitionsBuilder(),
        child: const SizedBox.shrink(),
      );

      expect(page, isA<MaterialPage<void>>());
      expect((page as MaterialPage<void>).child, isNot(isA<SizedBox>()));
    });
  });
}

DustRouterConfig<TestRoute> _config(_TestRouter router) {
  return DustRouterConfig<TestRoute>(
    router: router,
    initialRoute: TestRoute.home,
    parseRoute: (uri) => switch (uri.path) {
      '/login' => TestRoute.login,
      '/admin' => TestRoute.admin,
      '/slow' => TestRoute.slow,
      '/fast' => TestRoute.fast,
      '/guarded-public' => TestRoute.guardedPublic,
      _ => TestRoute.home,
    },
    routeLocation: (route) => switch (route) {
      TestRoute.home => '/',
      TestRoute.login => '/login',
      TestRoute.admin => '/admin',
      TestRoute.slow => '/slow',
      TestRoute.fast => '/fast',
      TestRoute.guardedPublic => '/guarded-public',
    },
    requiresAuth: (route) =>
        route == TestRoute.admin || route == TestRoute.slow,
    resolveGuards: (route) =>
        route == TestRoute.admin ||
            route == TestRoute.slow ||
            route == TestRoute.guardedPublic
        ? [
            _CallbackGuard(
              router.guardForRoute ?? (_) async => router.guardResult,
            ),
          ]
        : const [],
    buildPage: (route) =>
        MaterialPage<void>(name: route.name, child: const SizedBox.shrink()),
    restoreStack: (route) => route == TestRoute.home
        ? [TestRoute.home]
        : [TestRoute.home, route],
  );
}

Future<void> _flushAsync() async {
  await Future<void>.delayed(Duration.zero);
  await Future<void>.delayed(Duration.zero);
}

final class _TestRouter extends DustRouterBase<TestRoute> {
  _TestRouter({
    this.refreshListenable,
    this.guardResult = const RouteGuardResult.allow(),
    this.guardForRoute,
  });

  @override
  final Listenable? refreshListenable;

  RouteGuardResult<TestRoute> guardResult;
  Future<RouteGuardResult<TestRoute>> Function(TestRoute route)? guardForRoute;
  bool redirectToLogin = false;

  @override
  TestRoute? redirect(DustRouteState<TestRoute> state) {
    if (redirectToLogin && state.route != TestRoute.login) {
      return TestRoute.login;
    }
    return null;
  }
}

final class _CallbackGuard implements RouteGuard<TestRoute> {
  const _CallbackGuard(this.callback);

  final Future<RouteGuardResult<TestRoute>> Function(TestRoute route) callback;

  @override
  Future<RouteGuardResult<TestRoute>> canActivate(
    DustRouteState<TestRoute> state,
  ) {
    return callback(state.route);
  }
}
