# Routing

Dust routing generates a typed Flutter Navigator 2.0 router from annotations.
It does not depend on `go_router`, `auto_route`, or another routing package.

> Routing is currently planned and prototyped. The generated shapes shown here
> are the implementation target for the Rust routing plugin.

## Install

Add the router runtime package from this repository:

```yaml
dependencies:
  dust_router:
    path: ../path/to/dust/packages/dust_router
```

Generate code with Dust:

```bash
dust build
```

## Router Entrypoint

Create one routing entrypoint, usually `lib/route.dart`:

```dart
import 'package:flutter/material.dart' hide Route, Router;
import 'package:dust_router/dust_router.dart';

import 'app_state.dart';
import 'pages/dashboard_page.dart';
import 'pages/not_found_page.dart';
import 'route.g.dart';

export 'route.g.dart';
export 'package:dust_router/dust_router.dart';

@Router(
  initial: DashboardPage,
  notFound: NotFoundPage,
  refreshListenable: 'session',
)
final class AppRouter extends $AppRouter {
  AppRouter({required this.session});

  final AppSession session;

  @override
  AppRoutePath? redirect(RouteState state) {
    if (!session.isLoggedIn && state.route.requiresAuth) {
      return LoginRoute(from: state.location);
    }
    return null;
  }
}
```

The generated `route.g.dart` file imports annotated pages automatically. The app
should not manually import every routed page into `route.dart`.

## App Setup

Use the generated router with Flutter's `MaterialApp.router`:

```dart
final session = AppSession();

MaterialApp.router(
  routerConfig: AppRouter(session: session).config,
);
```

## Page Routes

Annotate normal pages or widgets in normal `lib/` files:

```dart
import 'package:flutter/material.dart' hide Route;
import 'package:dust_router/dust_router.dart';

import '../layout/app_shell.dart';

@Route('/', name: 'dashboard', shell: AppShell)
final class DashboardPage extends StatelessWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Text('Dashboard');
  }
}
```

Path parameters come from `:param` segments:

```dart
@Route('/projects/:projectId', name: 'project')
final class ProjectPage extends StatelessWidget {
  const ProjectPage({
    super.key,
    required this.projectId,
    this.tab,
  });

  final int projectId;
  final String? tab;

  @override
  Widget build(BuildContext context) {
    return Text('Project $projectId');
  }
}
```

`projectId` is read from the path. `tab` is a query parameter, so the generated
location can be `/projects/42?tab=activity`.

## Guards

Attach route-specific guards to a page:

```dart
@Route('/admin', name: 'admin', guards: [AdminGuard], shell: AppShell)
final class AdminPage extends StatelessWidget {
  const AdminPage({super.key});
}
```

Implement guards in `route.dart` or another imported file:

```dart
final class AdminGuard implements RouteGuard<AppRoutePath> {
  const AdminGuard(this.session);

  final AppSession session;

  @override
  Future<RouteGuardResult<AppRoutePath>> canActivate(RouteState state) async {
    if (!session.isAdmin) {
      return RouteGuardResult.redirect(const ForbiddenRoute());
    }
    return RouteGuardResult.allow();
  }
}
```

The router-level redirect runs before route guards. Guards may allow navigation
or return a typed redirect route.

## Shell Routes

Use a shell when a group of pages shares layout:

```dart
@Route('/billing/invoices/:invoiceId', name: 'invoice', shell: AppShell)
final class InvoicePage extends StatelessWidget {
  const InvoicePage({super.key, required this.invoiceId});

  final String invoiceId;
}
```

Dust groups nested paths automatically. Users do not need to create synthetic
parent pages for `/billing` or `/billing/invoices` unless those pages should be
real routes.

## Transitions

Use Flutter `PageTransitionsBuilder` values directly. Dust has no custom
transition registry:

```dart
@Route(
  '/checkout/:plan',
  name: 'checkout',
  transition: BottomToTopPageTransitionsBuilder(),
  fullscreenDialog: true,
)
final class CheckoutPage extends StatelessWidget {
  const CheckoutPage({super.key, required this.plan, this.annual = false});

  final String plan;
  final bool annual;
}
```

Omit `transition` to use the app-wide `PageTransitionsTheme`.

```dart
@Route('/admin', transition: ZoomPageTransitionsBuilder())
@Route('/login', transition: CupertinoPageTransitionsBuilder())
@Route('/dashboard')
```

## Navigation

Generated helpers are available from `BuildContext`:

```dart
context.routes.project(projectId: 42, tab: 'activity').go();
context.routes.login().push();
context.routes.dashboard().replace();
```

Dust generates one route factory per route name. Navigation actions are shared
on the returned object, so autocomplete stays route-name-first even when an app
has many routes.

You can also navigate with route objects:

```dart
DustRouter.of<AppRoutePath>(context).go(
  const CheckoutRoute(plan: 'team', annual: true),
);
```

## Deep Links And Web

The generated parser handles browser URLs and direct deep links:

```text
/invite/abc
/projects/42?tab=activity
/billing/invoices/inv_001
/search?q=dust&page=2
```

Unknown paths resolve to the configured not-found route.

## Supported Route Parameters

Route parameters are URL primitives only:

- `String`
- `int`
- `double`
- `bool`
- nullable variants
- constructor defaults

Complex values are not supported as route params:

- custom objects
- lists
- maps
- records
- functions
- `dynamic`

Load complex data from app state or a repository after the page is rebuilt from
the URL:

```dart
@Route('/projects/:projectId')
final class ProjectPage extends StatelessWidget {
  const ProjectPage({super.key, required this.projectId});

  final int projectId;

  @override
  Widget build(BuildContext context) {
    final project = context.projects.watch(projectId);
    return ProjectDetails(project: project);
  }
}
```

## Uri Encoding Rule

Generated routing code uses Dart `Uri` as the only encode/decode boundary, but
route classes should call a generated helper instead of manually starting every
path with an empty segment.

The helper owns the absolute-path detail:

```dart
String _routePath(
  List<String> segments, {
  Map<String, String>? queryParameters,
}) {
  return Uri(
    pathSegments: ['', ...segments],
    queryParameters: queryParameters,
  ).toString();
}
```

Location builders use `_routePath(...)`:

```dart
String get location => _routePath(
      ['projects', '$projectId'],
      queryParameters: {
        if (tab != null) 'tab': tab!,
      },
    );
```

Parsers read from `uri.pathSegments` and `uri.queryParameters`:

```dart
final projectId = int.tryParse(uri.pathSegments[1]);
if (projectId == null) {
  return _notFoundRoute(uri);
}

return ProjectRoute(
  projectId: projectId,
  tab: uri.queryParameters['tab'],
);
```

The helper creates an absolute path like `/projects/42` while still letting
`Uri` encode each segment. The Rust generator enforces supported param types
before Dart emission. It must not generate JSON fallback parsing, `dynamic`
parsing, or object codecs for route parameters.
