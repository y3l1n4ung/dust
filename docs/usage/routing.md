# Typed Routing

Dust Routing generates a type-safe Flutter Navigator 2.0 router from normal app
widgets. The generated `route.g.dart` file is standalone, owns page imports, and
is imported by the hand-written `lib/route.dart` entrypoint.

## Install

```yaml
dependencies:
  dust_flutter: ^0.1.0
```

## Router Entrypoint

```dart
import 'package:dust_flutter/route.dart';
import 'package:myapp/state/auth_view_model.dart';
import 'route.g.dart';

export 'route.g.dart';
export 'package:dust_flutter/route.dart';

@Router(initial: '/', notFound: '/404')
final class AppRouter extends $AppRouter {
  AppRouter({required this.auth});

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
the generated refresh source. Do not pass a string field name in `@Router`.

## Routes

```dart
@Route('/', name: 'home', shell: AppShell, guards: [AuthGuard])
final class HomePage extends StatelessWidget {
  const HomePage({super.key});
}

@Route('/models/:id', name: 'modelDetail', shell: AppShell)
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

@Route('/404', name: 'notFound', guards: [])
final class NotFoundPage extends StatelessWidget {
  const NotFoundPage({super.key, this.path = ''});

  final String path;
}
```

Route paths are absolute. Path segments like `:id` map to required constructor
parameters. Nullable or defaulted non-path constructor parameters become query
parameters.

## App Setup

```dart
MaterialApp.router(
  routerConfig: AppRouter(auth: authViewModel).config,
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
guards run first, async guards run second, preserving declaration order inside
each group.

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
