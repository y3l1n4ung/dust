# Typed Routing

Dust Routing generates a type-safe Flutter Navigator 2.0 router from normal app
widgets. The generated `route.g.dart` file is standalone, owns page imports, and
is imported by the hand-written `lib/route.dart` entrypoint.

## Install

```yaml
dependencies:
  dust_flutter: ^0.1.0
```

## Import Pattern

In the router entrypoint, import the routing package and export the generated API
for route pages:

```dart
import 'package:dust_flutter/route.dart';

import 'route.g.dart';

export 'route.g.dart';
export 'package:dust_flutter/route.dart';
```

In route page files, import your app router entrypoint:

```dart
import '../../route.dart';
```

## Router Entrypoint

```dart
import 'package:dust_flutter/route.dart';
import 'package:myapp/state/auth_view_model.dart';
import 'route.g.dart';

export 'route.g.dart';
export 'package:dust_flutter/route.dart';

@AppRouter(initial: '/', notFound: '/404')
final class RootRouter extends $RootRouter {
  RootRouter({required this.auth});

  final AuthViewModel auth;

  @override
  AppRoutePath? redirect(AppRoutePath route) {
    if (!auth.isLoggedIn && route.requiresAuth) {
      return LoginRoute(from: route.location);
    }
    if (auth.isLoggedIn && route is LoginRoute) {
      return const HomeRoute();
    }
    return null;
  }
}
```

Dust auto-discovers exactly one `Listenable`-like field on the router, such as a
`ViewModel`, `Listenable`, `ChangeNotifier`, or `ValueNotifier`, and wires it as
the generated refresh source. Do not pass a string field name in `@AppRouter`.

## Routes

```dart
@AppRoute('/', name: 'home', shell: AppShell, guards: [AuthGuard])
final class HomePage extends StatelessWidget {
  const HomePage({super.key});
}

@AppRoute('/models/:id', name: 'modelDetail', shell: AppShell)
final class ModelDetailPage extends StatelessWidget {
  const ModelDetailPage({
    super.key,
    required this.id,
    this.tab,
    this.archived,
  });

  final int id;
  final String? tab;
  final bool? archived;
}

@AppRoute('/404', name: 'notFound', guards: [])
final class NotFoundPage extends StatelessWidget {
  const NotFoundPage({super.key, this.path = ''});

  final String path;
}
```

Route paths are absolute. Path segments like `:id` map to required constructor
parameters. Nullable or defaulted non-path constructor parameters become query
parameters.

## Auth Semantics

Routes are protected by default. If a route omits `guards:`, its generated
`requiresAuth` getter returns `true`, so app-level router redirects can require
authentication before opening it.

Use `guards: []` to mark a route as public. Dust treats the empty guard list as
an explicit public-route decision and generates `requiresAuth => false` for that
route. Use this for login, register, not-found, invite, or other routes that
must remain reachable before auth succeeds.

Routes with one or more guards stay protected and run their guard chain after
the router-level redirect check.

> [!IMPORTANT]
> Use router-level `redirect` for global auth state: unauthenticated users,
> login/register bounce-back, expired sessions, and safe redirect-path handling.
> Use route guards for route-local permissions after the user is known, such as
> admin, staff, plan, tenant, or feature access.

> [!NOTE]
> When the router owns an auth `ViewModel`, `ChangeNotifier`, `ValueNotifier`, or
> other `Listenable` field, Dust wires it as the router refresh source. Emitting
> a new auth state rechecks the current route. For example, if a token expires
> while the user is on `AdminRoute`, the next auth state should make `redirect`
> return a `LoginRoute(redirectPath: route.location)`.

> [!TIP]
> Add one runtime routing test that starts on a protected route, changes the auth
> view model to an expired or unauthenticated state, drains the router refresh,
> and asserts that the current route is the expected login route with the
> original `redirectPath`.

## App Setup

```dart
MaterialApp.router(
  routerConfig: RootRouter(auth: authViewModel).config,
);
```

## Navigation

```dart
context.navigator.home().go();
context.navigator.modelDetail(id: 42, tab: 'perf').push();
context.navigator.login().replace();
context.navigator.pop();
```

Dust generates one method per route name. Each method returns `RouteAction<R>`
with `go`, `push`, and `replace`.

## Guards

```dart
final class AuthGuard implements RouteGuard<AppRoutePath> {
  const AuthGuard(this.auth);

  final AuthViewModel auth;

  @override
  AppRoutePath? canActivate(AppRoutePath route) {
    return auth.isLoggedIn ? null : LoginRoute(from: route.location);
  }
}
```

```dart
final class PermissionGuard implements AsyncRouteGuard<AppRoutePath> {
  const PermissionGuard(this.repo);

  final PermissionRepo repo;

  @override
  Future<AppRoutePath?> canActivate(AppRoutePath route) async {
    final allowed = await repo.canOpen(route);
    return allowed ? null : const ForbiddenRoute();
  }
}
```

A guard returns `null` to allow navigation or another route to redirect. Sync
and async guards run strictly in the generated declaration order.

> [!IMPORTANT]
> Guard constructor dependencies are injected from router fields by type. If a
> guard declares `const AdminGuard(this.auth)` and the router has exactly one
> `AuthViewModel auth` field, generated code passes that field. Generation fails
> when the dependency type cannot be resolved, is missing from the router, or is
> ambiguous because multiple router fields have the same type.

> [!TIP]
> In tests, resolve guards through generated `routeGuards(route, router)` and
> assert injected dependencies with `same(router.auth)`. That proves the guard is
> using the live app auth view model, not a separately constructed test object.

> [!NOTE]
> Keep guard decisions side-effect-light. Guards should read current state and
> return a redirect route; auth mutation such as token refresh, logout, or
> session restore belongs in the auth view model. That keeps route refresh
> behavior predictable and testable.

> [!WARNING]
> Client route guards are a navigation and UX boundary, not the final security
> boundary. Enforce admin, staff, tenant, and plan permissions on the server too;
> the app should mirror server-issued claims or profile permissions so navigation
> matches backend authorization.

## Parameters

Supported route parameter types are URL primitives only:

- `String`
- `int`
- `double`
- `bool`
- nullable variants
- query-parameter defaults

Unsupported route parameter types are rejected during generation.

## Deep Links

| URL | Resolved route |
| :--- | :--- |
| `/` | `HomeRoute()` |
| `/models/42?tab=perf` | `ModelDetailRoute(id: 42, tab: 'perf')` |
| `/404?path=%2Fbad%2Fpath` | `NotFoundRoute(path: '/bad/path')` |
