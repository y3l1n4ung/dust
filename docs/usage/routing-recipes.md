# Routing Recipes

Most apps only need a small set of route patterns. Keep one `@AppRouter`, then
compose these cases with normal `@AppRoute` options.

These recipes are based on the routing patterns covered by Flutter's Router
and deep-linking guidance, go_router's shell and stateful-shell APIs,
AutoRoute's deep-link handling, and Flutter's nested-flow and route-result
cookbook recipes:

| Researched pattern | Dust mapping |
| --- | --- |
| Router-driven deep links and browser URL sync | `MaterialApp.router(routerConfig: RootRouter().config)` |
| Shared layout around matched child routes | `@AppRoute(..., shell: AppShell)` |
| Deep-link stack restoration from prefix routes | Parent and child `@AppRoute` paths such as `/products` and `/products/:id` |
| Returning a value from a pushed page | `@AppRoute(..., result: Type)` plus `context.navigator.pop(value)` |
| Multi-step flows | Model shareable steps as normal typed routes under one path prefix |
| Web hash vs path URLs | Flutter's `usePathUrlStrategy()` before `runApp` |
| Stateful bottom-tab branch stacks | `@AppRoute(..., shell: AppShell, branch: 'mainTabs')` |

- Flutter navigation overview: <https://docs.flutter.dev/ui/navigation>
- Flutter nested navigation flow: <https://docs.flutter.dev/cookbook/effects/nested-nav>
- Flutter route results: <https://docs.flutter.dev/cookbook/navigation/returning-data>
- Flutter web URL strategy: <https://docs.flutter.dev/ui/navigation/url-strategies>
- go_router `ShellRoute`: <https://pub.dev/documentation/go_router/latest/go_router/ShellRoute-class.html>
- go_router `StatefulShellRoute`: <https://pub.dev/documentation/go_router/latest/go_router/StatefulShellRoute-class.html>
- AutoRoute deep linking: <https://pub.dev/packages/auto_route#deep-linking>

## Public Login With Redirect Back

```dart
@AppRoute('/login', name: 'login', guards: [])
final class LoginPage extends StatelessWidget {
  const LoginPage({this.redirectPath, super.key});

  final String? redirectPath;
}

@AppRouter(initial: '/', notFound: '/404')
final class RootRouter extends RootRouterBase {
  RootRouter({required this.auth});

  final AuthViewModel auth;

  @override
  AppRoutePath? redirect(AppRoutePath route) {
    if (!auth.state.isAuthenticated && route.requiresAuth) {
      return LoginRoute(redirectPath: route.location);
    }
    return null;
  }
}
```

Call chain after a private deep link:

```text
browser opens /orders/42
RootRouter.redirect(OrderRoute(id: 42))
LoginRoute(redirectPath: /orders/42)
login succeeds
context.navigator.order(id: 42).go()
```

## Search Page With Shareable Filters

```dart
@AppRoute('/products', name: 'productSearch', guards: [])
final class ProductSearchPage extends StatelessWidget {
  const ProductSearchPage({
    this.query,
    this.page = 1,
    this.showArchived = false,
    super.key,
  });

  final String? query;
  final int page;
  final bool showArchived;
}
```

Generated calls:

```dart
context.navigator
    .productSearch(query: 'tea', page: 2, showArchived: true)
    .go();
```

This produces a URL like `/products?query=tea&page=2&showArchived=true`.

## Public Invite Or Magic Link

```dart
@AppRoute('/invite/:code', name: 'invite', guards: [])
final class InvitePage extends StatelessWidget {
  const InvitePage({required this.code, this.team, super.key});

  final String code;
  final String? team;
}
```

Generated calls:

```dart
context.navigator.invite(code: 'A1B2C3', team: 'design').go();
```

Use route guards or the page's ViewModel to validate the token with your
backend. Keep the route public so signed-out users can open it from email.

## Organization-Scoped Detail Page

```dart
@AppRoute(
  '/orgs/:orgId/projects/:projectId',
  name: 'orgProject',
)
final class OrgProjectPage extends StatelessWidget {
  const OrgProjectPage({
    required this.orgId,
    required this.projectId,
    this.tab,
    super.key,
  });

  final String orgId;
  final int projectId;
  final String? tab;
}
```

Generated calls:

```dart
context.navigator
    .orgProject(orgId: 'acme', projectId: 42, tab: 'activity')
    .push();
```

This keeps tenant or workspace context in the path, which makes browser reloads,
external links, and analytics easier to reason about.

## Dashboard Shell With Child Pages

```dart
final class DashboardShell extends StatelessWidget {
  const DashboardShell({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) => Scaffold(body: child);
}

@AppRoute('/dashboard', name: 'dashboard', shell: DashboardShell)
final class DashboardPage extends StatelessWidget {}

@AppRoute('/dashboard/orders', name: 'dashboardOrders')
final class DashboardOrdersPage extends StatelessWidget {}
```

Call chain:

```text
context.navigator.dashboardOrders().go()
DashboardOrdersRoute()
DashboardShell(child: DashboardOrdersPage())
Navigator.pages = [/dashboard/orders]
```

No extra shell annotation or marker class is needed.

## Stateful Tab Branches

Use `branch:` when routes should keep independent tab stacks. Keep `shell:` for
layout and `branch:` for navigation state:

```dart
@AppRoute(
  '/tabs/home',
  name: 'tabHome',
  shell: DashboardShell,
  branch: 'mainTabs',
)
final class TabHomePage extends StatelessWidget {}

@AppRoute('/tabs/home/details', name: 'tabHomeDetails')
final class TabHomeDetailsPage extends StatelessWidget {}

@AppRoute(
  '/tabs/orders',
  name: 'tabOrders',
  shell: DashboardShell,
  branch: 'ordersTabs',
)
final class TabOrdersPage extends StatelessWidget {}
```

Call chain:

```text
context.navigator.tabHomeDetails().push()
TabHomeDetailsRoute()
branch mainTabs keeps [/tabs/home, /tabs/home/details]
context.navigator.tabOrders().go()
branch ordersTabs becomes active
context.navigator.tabHome().go()
branch mainTabs is restored with [/tabs/home, /tabs/home/details]
```

Child paths inherit the nearest parent `branch:` just like they inherit
`shell:`. Dust also generates a stable constant for each branch value, for
example `rootBranchMainTabs`, and reuses that constant in route metadata and
debug helpers.

## Multi-Step Setup Flow

Use normal Dust routes for flow steps that should be shareable, reloadable, or
observable. A shell gives the flow one layout without making app code touch
`Navigator` directly.

```dart
final class SetupShell extends StatelessWidget {
  const SetupShell({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) => Scaffold(body: child);
}

@AppRoute('/setup', name: 'setup', guards: [], shell: SetupShell)
final class SetupPage extends StatelessWidget {
  const SetupPage({super.key});
}

@AppRoute('/setup/connect', name: 'setupConnect')
final class SetupConnectPage extends StatelessWidget {
  const SetupConnectPage({super.key});
}
```

Generated calls:

```dart
context.navigator.setup().go();
context.navigator.setupConnect().replace();
```

Dust then restores `/setup/connect` as the setup shell plus the connect step.
This keeps the flow URL-friendly without requiring handwritten nested
`Navigator` code in the app.

## Web Path URLs

Flutter web defaults to hash URLs. If your app should use normal path URLs,
configure Flutter's path strategy before creating the router:

```dart
import 'package:flutter_web_plugins/url_strategy.dart';

void main() {
  usePathUrlStrategy();

  final router = RootRouter();
  runApp(MaterialApp.router(routerConfig: router.config));
}
```

Your web server must rewrite unknown app paths to `index.html`, otherwise a
browser refresh on `/products/42` can miss the Flutter app entirely.

For host-specific rewrite examples, subdirectory deploys, and deployment
verification steps, see the
[router web URL deployment guide](./routing-web-deployment.md).

## Picker Or Dialog Route With Result

```dart
@AppRoute(
  '/product-picker',
  name: 'productPicker',
  result: int,
  guards: [],
  transition: BottomToTopPageTransitionsBuilder(),
  fullscreenDialog: true,
)
final class ProductPickerPage extends StatelessWidget {
  const ProductPickerPage({super.key});
}
```

Generated calls:

```dart
final productId = await context.navigator.productPicker().push();
if (productId != null) {
  context.navigator.product(id: productId).go();
}
```

Inside the picker:

```dart
context.navigator.pop(selectedProductId);
```

## Guarded Admin Page

```dart
final class AdminGuard implements RouteGuard<AppRoutePath> {
  const AdminGuard(this.auth);

  final AuthViewModel auth;

  @override
  AppRoutePath? canActivate(AppRoutePath route) {
    return auth.state.isAdmin ? null : const HomeRoute();
  }
}

@AppRoute('/admin', name: 'admin', guards: [AdminGuard])
final class AdminPage extends StatelessWidget {
  const AdminPage({super.key});
}
```

Call chain:

```text
context.navigator.admin().push()
RootRouter.redirect(AdminRoute())
AdminGuard.canActivate(AdminRoute())
Navigator stack commit or HomeRoute redirect
```

## Analytics And Error Reporting

```dart
@AppRouter(initial: '/', notFound: '/404')
final class RootRouter extends RootRouterBase {
  @override
  void didChangeRouteStack(
    RouteStack<AppRoutePath> previous,
    RouteStack<AppRoutePath> next,
  ) {
    analytics.screenView(next.last.location);
  }

  @override
  void onException(Object error, StackTrace stackTrace) {
    errorReporter.capture(error, stackTrace);
  }
}
```

Use this for route stack analytics and unawaited navigation failures such as
redirect cycles.
