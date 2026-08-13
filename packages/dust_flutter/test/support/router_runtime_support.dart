import 'package:dust_flutter/route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

RouterRuntimeConfig<TestRoute> runtimeConfig({
  RouterBase<TestRoute>? router,
  RouteGuardResolver<TestRoute>? resolveGuards,
  RouteStackRestorer<TestRoute>? restoreStack,
  Widget Function(BuildContext context, TestRoute route)? buildChild,
  PageTransitionsBuilder? Function(TestRoute route)? transitionForRoute,
}) {
  return RouterRuntimeConfig<TestRoute>(
    router: router ?? NoRedirectRouter(),
    initialRoute: const TestRoute('/safe'),
    parseRoute: (uri) => TestRoute(uri.toString()),
    routeLocation: (route) => route.location,
    requiresAuth: (_) => false,
    routeBranch: (route) => switch (route.location) {
      '/branch' => 'mainTabs',
      _ => null,
    },
    resolveGuards: resolveGuards ?? (_) => const [],
    restoreStack: restoreStack,
    buildPage: (route, key, onPopInvoked) => generatedPage<Object?>(
      key: key,
      location: route.location,
      name: route.location,
      onPopInvoked: onPopInvoked,
      transition: transitionForRoute?.call(route),
      child: Builder(
        builder: (context) =>
            buildChild?.call(context, route) ?? const SizedBox(),
      ),
    ),
    debugRoutes: const [
      GeneratedRoute('/safe', page: SizedBox, name: 'safe'),
      GeneratedRoute('/private', page: SizedBox, name: 'private'),
      GeneratedRoute(
        '/branch',
        page: SizedBox,
        name: 'branch',
        shell: DebugShell,
        branch: 'mainTabs',
      ),
      GeneratedRoute('/guard-private', page: SizedBox, name: 'guardPrivate'),
      GeneratedRoute('/login', page: SizedBox, name: 'login'),
      GeneratedRoute(
        '/nested',
        routes: [
          GeneratedRoute(':id', page: SizedBox, name: 'nestedDetail'),
        ],
      ),
    ],
    debugInfo: (route) => switch (route.location) {
      '/safe' => const RouteDebugInfo(name: 'safe'),
      '/private' => const RouteDebugInfo(name: 'private'),
      '/branch' => const RouteDebugInfo(
          name: 'branch',
          shell: 'DebugShell',
          branch: 'mainTabs',
        ),
      '/guard-private' => const RouteDebugInfo(name: 'guardPrivate'),
      '/login' => const RouteDebugInfo(name: 'login'),
      _ => const RouteDebugInfo(),
    },
  );
}

Future<void> pumpRuntimeRouter(
  WidgetTester tester,
  GeneratedRouterDelegate<TestRoute> delegate,
) async {
  await tester.pumpWidget(
    MaterialApp.router(
      routerConfig: RouterConfig<TestRoute>(
        routeInformationProvider: PlatformRouteInformationProvider(
          initialRouteInformation: RouteInformation(uri: Uri.parse('/safe')),
        ),
        routeInformationParser: GeneratedRouteInformationParser<TestRoute>(
          parseRoute: (uri) => TestRoute(uri.toString()),
          routeLocation: (route) => route.location,
        ),
        routerDelegate: delegate,
        backButtonDispatcher: RootBackButtonDispatcher(),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

final class TestRoute {
  const TestRoute(this.location);

  final String location;
}

final class DebugShell extends StatelessWidget {
  const DebugShell({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) => child;
}

final class RecordingPageTransitionsBuilder extends PageTransitionsBuilder {
  int calls = 0;
  final List<String?> routeNames = [];

  @override
  Widget buildTransitions<T>(
    PageRoute<T> route,
    BuildContext context,
    Animation<double> animation,
    Animation<double> secondaryAnimation,
    Widget child,
  ) {
    calls += 1;
    routeNames.add(route.settings.name);
    return child;
  }
}

final class NoRedirectRouter extends RouterBase<TestRoute> {}

final class RecordingRouter extends RouterBase<TestRoute> {
  final stackChanges = <String>[];

  @override
  void didChangeRouteStack(
    RouteStack<TestRoute> previous,
    RouteStack<TestRoute> next,
  ) {
    stackChanges.add('${locations(previous)} => ${locations(next)}');
  }
}

final class RefreshRouter extends RouterBase<TestRoute> {
  final refreshNotifier = ChangeNotifier();

  @override
  Listenable get refreshListenable => refreshNotifier;
}

final class AuthRouter extends RouterBase<TestRoute> {
  bool isAuthenticated = true;

  @override
  TestRoute? redirect(TestRoute route) {
    if (!isAuthenticated && route.location == '/private') {
      return const TestRoute('/login');
    }
    return null;
  }
}

final class ExceptionRecordingRouter extends RouterBase<TestRoute> {
  final errors = <Object>[];
  final stackTraces = <StackTrace>[];

  @override
  TestRoute? redirect(TestRoute route) {
    return switch (route.location) {
      '/one' => const TestRoute('/two'),
      '/two' => const TestRoute('/one'),
      _ => null,
    };
  }

  @override
  void onException(Object error, StackTrace stackTrace) {
    errors.add(error);
    stackTraces.add(stackTrace);
  }
}

final class ObserverRouter extends RouterBase<TestRoute> {
  ObserverRouter(this._observers);

  final List<NavigatorObserver> _observers;

  @override
  List<NavigatorObserver> get observers => _observers;
}

final class RecordingNavigatorObserver extends NavigatorObserver {
  final pushed = <String?>[];
  final popped = <String?>[];

  @override
  void didPush(Route<dynamic> route, Route<dynamic>? previousRoute) {
    pushed.add(route.settings.name);
  }

  @override
  void didPop(Route<dynamic> route, Route<dynamic>? previousRoute) {
    popped.add(route.settings.name);
  }

  void clear() {
    pushed.clear();
    popped.clear();
  }
}

final class DebugRouter extends RouterBase<TestRoute> {
  @override
  bool get debugLogDiagnostics => true;

  @override
  TestRoute? redirect(TestRoute route) {
    return route.location == '/private' ? const TestRoute('/login') : null;
  }
}

final class RouterRedirectCycle extends RouterBase<TestRoute> {
  @override
  TestRoute? redirect(TestRoute route) {
    return switch (route.location) {
      '/one' => const TestRoute('/two'),
      '/two' => const TestRoute('/one'),
      _ => null,
    };
  }
}

final class LoginGuard implements RouteGuard<TestRoute> {
  const LoginGuard();

  @override
  TestRoute? canActivate(TestRoute route) => const TestRoute('/login');
}

final class GuardRedirectCycle implements RouteGuard<TestRoute> {
  @override
  TestRoute? canActivate(TestRoute route) {
    return switch (route.location) {
      '/guard-one' => const TestRoute('/guard-two'),
      '/guard-two' => const TestRoute('/guard-one'),
      _ => null,
    };
  }
}

final class BlockingGuard implements AsyncRouteGuard<TestRoute> {
  const BlockingGuard(this.result);

  final Future<TestRoute?> result;

  @override
  Future<TestRoute?> canActivate(TestRoute route) => result;
}

final class RecordingGuard implements RouteGuard<TestRoute> {
  const RecordingGuard(this.location, this.calls);

  final String location;
  final List<String> calls;

  @override
  TestRoute? canActivate(TestRoute route) {
    calls.add(location);
    return null;
  }
}

final class RedirectRecordingGuard implements RouteGuard<TestRoute> {
  const RedirectRecordingGuard(this.location, this.calls, this.redirect);

  final String location;
  final List<String> calls;
  final TestRoute redirect;

  @override
  TestRoute? canActivate(TestRoute route) {
    calls.add(location);
    return redirect;
  }
}

String locations(RouteStack<TestRoute> stack) {
  return '[${stack.map((route) => route.location).join(', ')}]';
}
