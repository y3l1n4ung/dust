import 'package:dust_flutter/route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('controller can push and pop with a typed result', (
    tester,
  ) async {
    late Future<String?> pushResult;
    final delegate = GeneratedRouterDelegate<_Route>(
      _runtimeConfig(
        buildChild: (context, route) {
          if (route.location == '/detail') {
            return TextButton(
              onPressed: () {
                RouterController.of<_Route>(context).pop('done');
              },
              child: const Text('finish detail'),
            );
          }

          return TextButton(
            onPressed: () {
              pushResult = RouterController.of<_Route>(
                context,
              ).push<String>(const _Route('/detail'));
            },
            child: const Text('open detail'),
          );
        },
      ),
    );

    await tester.pumpWidget(_routerApp(delegate));
    await tester.pumpAndSettle();

    await tester.tap(find.text('open detail'));
    await tester.pumpAndSettle();

    expect(delegate.stack.map((route) => route.location), [
      '/safe',
      '/detail',
    ]);

    await tester.tap(find.text('finish detail'));
    await tester.pumpAndSettle();

    expect(delegate.stack.map((route) => route.location), ['/safe']);
    await expectLater(pushResult, completion('done'));
  });

  testWidgets('controller exposes an immutable stack snapshot', (
    tester,
  ) async {
    late RouterController<_Route> controller;
    final delegate = GeneratedRouterDelegate<_Route>(
      _runtimeConfig(
        buildChild: (context, route) {
          controller = RouterController.of<_Route>(context);
          return Text(route.location);
        },
      ),
    );

    await tester.pumpWidget(_routerApp(delegate));
    await tester.pumpAndSettle();

    expect(controller.currentRoute.location, '/safe');
    expect(controller.stack.map((route) => route.location), ['/safe']);
    expect(
      () => controller.stack.add(const _Route('/detail')),
      throwsUnsupportedError,
    );
  });
}

Widget _routerApp(GeneratedRouterDelegate<_Route> delegate) {
  return MaterialApp.router(
    routerConfig: RouterConfig<_Route>(
      routeInformationProvider: PlatformRouteInformationProvider(
        initialRouteInformation: RouteInformation(uri: Uri.parse('/safe')),
      ),
      routeInformationParser: GeneratedRouteInformationParser<_Route>(
        parseRoute: (uri) => _Route(uri.toString()),
        routeLocation: (route) => route.location,
      ),
      routerDelegate: delegate,
      backButtonDispatcher: RootBackButtonDispatcher(),
    ),
  );
}

RouterRuntimeConfig<_Route> _runtimeConfig({
  required Widget Function(BuildContext context, _Route route) buildChild,
}) {
  return RouterRuntimeConfig<_Route>(
    router: _NoRedirectRouter(),
    initialRoute: const _Route('/safe'),
    parseRoute: (uri) => _Route(uri.toString()),
    routeLocation: (route) => route.location,
    requiresAuth: (_) => false,
    routeBranch: (_) => null,
    resolveGuards: (_) => const [],
    buildPage: (route, key, onPopInvoked) {
      return MaterialPage<Object?>(
        key: key,
        name: route.location,
        onPopInvoked: onPopInvoked,
        child: Builder(builder: (context) => buildChild(context, route)),
      );
    },
  );
}

final class _Route {
  const _Route(this.location);

  final String location;
}

final class _NoRedirectRouter extends RouterBase<_Route> {}
