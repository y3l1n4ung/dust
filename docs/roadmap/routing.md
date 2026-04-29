# Router Plan

## Goal

Generate a production-ready Flutter router on top of Navigator 2.0 with no
dependency on `go_router`, `auto_route`, or a Dust runtime package. The complete
router framework must be self-contained in one generated `.g.dart` file.

URLs are the source of truth, route classes are strongly typed, pages are built
from generated route data, and navigation is explicit.

## Package Shape

- Dart annotation package: `dust_router_annotation`
- Rust plugin crate: `dust_router_plugin`
- No runtime package

The generated `app_router.g.dart` contains all runtime types needed by the app.

## Import Constraint

Generated files are Dart `part` files and cannot add imports. The router root
file must import Flutter, the annotation package, transition functions, guard
types, services, and every routed page file or a page barrel.

## Router Root API

```dart
import 'dart:async';

import 'package:flutter/material.dart';
import 'package:dust_router_annotation/dust_router_annotation.dart';

import 'auth_service.dart';
import 'analytics_service.dart';
import 'guards/auth_guard.dart';
import 'pages/home_page.dart';
import 'pages/login_page.dart';
import 'pages/user_detail_page.dart';
import 'transitions/app_transitions.dart';

part 'app_router.g.dart';

@DustRouter(
  initialLocation: '/',
  transition: AppTransitions.fade,
  transitionDuration: Duration(milliseconds: 250),
)
class AppRouter extends _$AppRouter {
  AppRouter({
    required this.auth,
    required this.analytics,
  });

  final AuthService auth;
  final AnalyticsService analytics;

  @override
  List<DustRouteGuard> get guards => [
        AuthGuard(auth),
      ];

  @override
  FutureOr<DustRouteData?> redirect(DustRouteData route) {
    if (auth.isLoggedIn && route is LoginRoute) {
      return const HomeRoute();
    }
    return null;
  }

  @override
  void onNavigate(DustRouteData route) {
    analytics.trackScreen(route.location);
  }
}
```

The router class is meaningful. It owns injected dependencies and overrides
generated hooks. The generated base class provides `config`, route parsing,
navigation, guards, redirects, and stack management.

## Page Route API

```dart
@DustRoute(path: '/', initial: true)
class HomePage extends StatelessWidget {
  const HomePage({super.key});
}

@DustRoute(
  path: '/users/:id',
  guards: [AuthGuard],
  transition: AppTransitions.slideFromRight,
  transitionDuration: Duration(milliseconds: 300),
)
class UserDetailPage extends StatelessWidget {
  final String id;
  final String? tab;
  final int page;

  const UserDetailPage({
    super.key,
    required this.id,
    this.tab,
    this.page = 1,
  });
}
```

Path parameters are inferred from `:id`. Constructor parameters not used by the
path become query parameters. No separate `@Path` or `@Query` annotation is
needed for routes.

## App Usage

```dart
final router = AppRouter(
  auth: authService,
  analytics: analyticsService,
);

MaterialApp.router(
  routerConfig: router.config,
);
```

## Generated Navigation API

```dart
context.goHome();

context.pushUserDetail(id: '123');

context.replaceUserDetail(
  id: '123',
  tab: 'posts',
  page: 2,
);

context.go(
  const UserDetailRoute(id: '123'),
);
```

Generated helper names strip the `Page` suffix:

- `HomePage` -> `HomeRoute`, `goHome`, `pushHome`, `replaceHome`
- `UserDetailPage` -> `UserDetailRoute`, `goUserDetail`,
  `pushUserDetail`, `replaceUserDetail`

## Generated File Contents

The single generated `app_router.g.dart` includes:

- `DustRouteData`
- `DustRouteGuard`
- `DustRouteTransitionBuilder`
- generated route data classes
- generated `NotFoundRoute`
- generated router controller
- generated `RouterDelegate`
- generated `RouteInformationParser`
- generated `RouterScope`
- generated `RouterConfig`
- generated `BuildContext` navigation extensions
- generated route records
- generated URL parse/build helpers
- generated transition page

## Generated Base Class Shape

```dart
abstract class _$AppRouter {
  late final RouterConfig<DustRouteData> config = RouterConfig<DustRouteData>(
    routeInformationParser: _DustRouteInformationParser(),
    routerDelegate: _DustRouterDelegate(router: this),
    backButtonDispatcher: RootBackButtonDispatcher(),
  );

  List<DustRouteGuard> get guards => const [];

  FutureOr<DustRouteData?> redirect(DustRouteData route) => null;

  void onNavigate(DustRouteData route) {}
}
```

## Route Data Rules

```dart
final class UserDetailRoute extends DustRouteData {
  const UserDetailRoute({
    required this.id,
    this.tab,
    this.page = 1,
  });

  final String id;
  final String? tab;
  final int page;

  @override
  String get location => _DustUri(
        pathSegments: ['users', id],
        queryParameters: {
          if (tab != null) 'tab': tab,
          if (page != 1) 'page': page.toString(),
        },
      ).toString();

  @override
  Widget build(BuildContext context) {
    return UserDetailPage(
      id: id,
      tab: tab,
      page: page,
    );
  }
}
```

Rules:

- `:id` maps to constructor field `id`.
- Path params must be required and non-nullable.
- Other constructor fields become query params.
- Nullable query params are omitted when `null`.
- Query params with defaults are omitted when value equals default.
- `Key? key` and `super.key` are ignored.
- Supported param types: `String`, `int`, `double`, `bool`, enum, and nullable
  variants.
- Unsupported custom objects, lists, maps, records, functions, and dynamic route
  params produce diagnostics.

## Controller Behavior

The generated controller owns the navigation stack and exposes:

- `go(DustRouteData route)`
- `push(DustRouteData route)`
- `replace(DustRouteData route)`
- `pop<T>([T? result])`

Behavior:

- URL parse produces route data.
- Redirect runs before stack mutation.
- Route-specific guards run after global redirect.
- Redirects are async-safe.
- Redirect loop depth is capped at 5.
- `go` replaces stack with the target route.
- `push` appends target route and updates URL to the top route.
- `replace` replaces the top route.
- Browser back/forward calls `setNewRoutePath` and treats URL as source of
  truth.
- `onNavigate` runs after successful stack mutation.
- Page keys are stable and derived from route location.

## Guards

```dart
abstract class DustRouteGuard {
  const DustRouteGuard();

  FutureOr<DustRouteData?> redirect(DustRouteData route);
}
```

Route-specific guards are declared by type:

```dart
@DustRoute(
  path: '/users/:id',
  guards: [AuthGuard],
)
class UserDetailPage extends StatelessWidget {}
```

The router provides instances:

```dart
@override
List<DustRouteGuard> get guards => [
      AuthGuard(auth),
    ];
```

If a route references `AuthGuard`, one matching instance must exist in
`router.guards`. Missing guard instances produce a navigation error route.

## Transitions

Dust does not provide a fixed transition enum. Users provide transition
builders.

```dart
typedef DustRouteTransitionBuilder = Widget Function(
  BuildContext context,
  Animation<double> animation,
  Animation<double> secondaryAnimation,
  Widget child,
);
```

Router-level default:

```dart
@DustRouter(
  initialLocation: '/',
  transition: AppTransitions.fade,
  transitionDuration: Duration(milliseconds: 250),
)
class AppRouter extends _$AppRouter {}
```

Route-level override:

```dart
@DustRoute(
  path: '/users/:id',
  transition: AppTransitions.slideFromRight,
  transitionDuration: Duration(milliseconds: 300),
)
class UserDetailPage extends StatelessWidget {}
```

Annotation values must be compile-time constants, so transition builders must be
top-level or static function tear-offs.

Generated `_DustTransitionPage` uses `MaterialPageRoute` when no transition
builder is provided and `PageRouteBuilder` when one is provided.

## Deep Links

The generated `RouteInformationParser` must:

- parse browser and mobile deep links into route data
- restore route data back to URL strings
- handle browser refresh
- support back and forward navigation
- route unknown paths to `NotFoundRoute`
- preserve query parameter defaults and omit default values from generated URLs

## Validation

Dust must emit diagnostics when:

- no `@DustRouter` exists for a generated router file
- a route page is not a `Widget`
- duplicate route paths exist
- multiple routes are marked `initial: true`
- no initial route exists and router has no `initialLocation`
- path syntax is invalid
- a path placeholder has no constructor field
- a path constructor field is nullable or optional
- a query param type is unsupported
- a query param is required but missing during deep link parsing
- a default query value cannot be parsed as a compile-time constant
- a guard type is not a subtype of `DustRouteGuard`
- a transition function has the wrong signature
- a route constructor cannot be called from generated route data

## Tests

- Rust parser tests for `@DustRouter` and `@DustRoute`.
- Rust lowering tests for path params, query params, defaults, guards, and
  transitions.
- Rust validation tests for every diagnostic above.
- Golden tests for generated router file.
- Flutter analyzer tests for generated route classes and helpers.
- Flutter widget tests for go, push, replace, pop, deep link, browser back,
  browser forward, unknown route, async redirect, guard redirect, and transition
  page creation.

## Done

- A Flutter app can instantiate `AppRouter` with dependencies and pass
  `router.config` to `MaterialApp.router`.
- Routes are strongly typed and generated from annotated page widgets.
- URL parsing and URL generation round-trip.
- Redirects and guards are async-safe and loop-limited.
- Transition builders are supported without a Dust runtime package.
- The complete framework lives inside one generated `.g.dart` file.
