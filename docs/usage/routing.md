# Typed Routing

Dust generates a typed Flutter Navigator 2.0 router from annotated widgets.
For a source-grounded comparison with go_router, AutoRoute, Beamer, and
hand-written Router 2.0, see the
[router DX comparison](./routing-dx-comparison.md).
For planned deep-link, web URL, branch, diagnostics, and tooling improvements,
see the [routing improvement roadmap](./routing-improvement-roadmap.md).

## Add the Package

Install the Dust CLI from the [root guide](../../README.md#installation), then
add the Flutter runtime:

```bash
flutter pub add dust_flutter
```

## Quick Start

Create one router entrypoint at `lib/route.dart`:

```dart
import 'package:dust_flutter/route.dart';

import 'route.g.dart';

export 'route.g.dart';
export 'package:dust_flutter/route.dart';

@AppRouter(initial: '/', notFound: '/404')
final class RootRouter extends $RootRouter {}
```

Import that entrypoint from each route page:

```dart
import 'package:flutter/material.dart';

import 'package:my_app/route.dart';

@AppRoute('/', name: 'home', guards: [])
final class HomePage extends StatelessWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context) => const Scaffold();
}

@AppRoute('/products/:id', name: 'product', guards: [])
final class ProductPage extends StatelessWidget {
  const ProductPage({super.key, required this.id, this.tab});

  final int id;
  final String? tab;

  @override
  Widget build(BuildContext context) => Scaffold(body: Text('Product $id'));
}

@AppRoute('/404', name: 'notFound', guards: [])
final class NotFoundPage extends StatelessWidget {
  const NotFoundPage({super.key, this.path = ''});

  final String path;

  @override
  Widget build(BuildContext context) => Scaffold(body: Text(path));
}
```

Generate the router and pass its configuration to Flutter:

```bash
dust build
```

```dart
void main() {
  final router = RootRouter();
  runApp(MaterialApp.router(routerConfig: router.config));
}
```

> [!IMPORTANT]
> Keep `route.dart` as the app-facing routing entrypoint. It imports and exports
> `route.g.dart`; route pages import `route.dart` to use annotations and
> generated navigation helpers.

## Navigation

Dust generates a method for each route name:

```dart
context.navigator.home().go();
await context.navigator.product(id: 42, tab: 'reviews').push();
context.navigator.product(id: 7).replace();
context.navigator.pop();
```

| Method | Stack behavior |
| :--- | :--- |
| `go()` | Replaces the whole stack with the target route. |
| `push()` | Adds the route and completes when it is popped or removed. |
| `replace()` | Replaces the current top route. |
| `pop()` | Pops the top route when possible. |

If `name` is omitted, Dust derives it from the widget class name. For example,
`ProductDetailsScreen` becomes `productDetails()` and
`ProductDetailsRoute`.

## Use Case Recipes

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

### Public Login With Redirect Back

```dart
@AppRoute('/login', name: 'login', guards: [])
final class LoginPage extends StatelessWidget {
  const LoginPage({this.redirectPath, super.key});

  final String? redirectPath;
}

@AppRouter(initial: '/', notFound: '/404')
final class RootRouter extends $RootRouter {
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

### Search Page With Shareable Filters

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

### Public Invite Or Magic Link

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

### Organization-Scoped Detail Page

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

### Dashboard Shell With Child Pages

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

### Stateful Tab Branches

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
`shell:`.

### Multi-Step Setup Flow

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

### Web Path URLs

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

### Picker Or Dialog Route With Result

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

### Guarded Admin Page

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

### Analytics And Error Reporting

```dart
@AppRouter(initial: '/', notFound: '/404')
final class RootRouter extends $RootRouter {
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

## Typed Route Results

Routes return `void` by default. Add `result: Type` when a pushed route should
return a value:

```dart
@AppRoute('/support/chat', name: 'supportChat', result: bool)
final class SupportChatScreen extends StatefulWidget {
  const SupportChatScreen({super.key});
}
```

The generated helper returns `Future<bool?>`:

```dart
final sentMessage = await context.navigator.supportChat().push();
if (sentMessage == true) {
  AppSnackbar.success(context, 'Support message sent');
}
```

Return the value through the generated navigator:

```dart
context.navigator.pop(true);
```

> [!TIP]
> Use route results for one-time UI answers such as pickers, confirmations, or
> form completion. Keep shareable state in path or query parameters.

## Route Parameters

Path parameters match required, non-nullable constructor parameters. Other
parameters must be nullable or have a default and are encoded as query values:

```dart
@AppRoute('/products/:id', name: 'product', guards: [])
final class ProductPage extends StatelessWidget {
  const ProductPage({
    super.key,
    required this.id,
    this.tab,
    this.preview = false,
  });

  final int id;
  final String? tab;
  final bool preview;

  @override
  Widget build(BuildContext context) => Scaffold(body: Text('Product $id'));
}
```

This route can produce `/products/42?tab=reviews&preview=true`. Default-valued
query parameters are omitted when their value matches the default.

Supported URL types are `String`, `int`, `double`, `bool`, and their nullable
variants. Route widgets need an unnamed generative constructor. Invalid typed
path values resolve to the configured not-found route.

## Deep Links and Browser URLs

The generated route information parser converts incoming platform and browser
URIs into typed routes. A deep link such as:

```text
/products/42?tab=reviews
```

becomes `ProductRoute(id: 42, tab: 'reviews')`. Dust also rebuilds the stack
from matching path prefixes, so `/orders/ORDER-9` can restore the initial page,
`/orders`, and the order detail page.

Unknown query values and URI fragments remain in `route.location`. This lets
auth redirects preserve campaign parameters, fragments, and other URI data
that the page does not model directly.

> [!TIP]
> Test both `parseAppRoute(Uri.parse(url))` and the generated route's
> `location`. This catches decoding and round-trip regressions before testing a
> full platform deep-link flow.

## Redirects and Authentication

Routes are protected by default. Add `guards: []` when a route must be public:

```dart
@AppRoute('/login', name: 'login', guards: [])
final class LoginPage extends StatelessWidget {
  const LoginPage({super.key, this.redirectPath});

  final String? redirectPath;

  @override
  Widget build(BuildContext context) => const Scaffold();
}
```

Use the router's `redirect` method for app-wide auth decisions:

```dart
@AppRouter(initial: '/', notFound: '/404')
final class RootRouter extends $RootRouter {
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

Dust uses the router's single `Listenable`, `ChangeNotifier`, `ValueNotifier`,
or `*ViewModel` field as its refresh source. When that object notifies, the
current route is redirected and guarded again. Generation fails if the router
has more than one such field.

> [!IMPORTANT]
> Validate saved redirect locations before reopening them. Reject external
> hosts and unknown routes instead of treating arbitrary input as an internal
> destination.

## Route Guards

Use guards for route-specific access checks:

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

  @override
  Widget build(BuildContext context) => const Scaffold();
}
```

A guard returns `null` to allow navigation or another route to redirect.
Implement `AsyncRouteGuard<AppRoutePath>` when the decision needs a `Future`.
Mixed sync and async guards run in annotation order.

Guard constructor dependencies are matched to router fields by type. In this
example, Dust passes `RootRouter.auth` to `AdminGuard`. Generation fails when a
required dependency is missing or ambiguous.

> [!NOTE]
> Route guards control navigation, not backend authorization. Enforce access on
> the server as well.

## Shells and Transitions

Dust keeps shell DX inside `@AppRoute`; there is no separate shell annotation.
A shell is just a Flutter widget that accepts a required named `Widget child`
argument:

```dart
final class AppShell extends StatelessWidget {
  const AppShell({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Scaffold(body: child);
  }
}

@AppRoute('/account', name: 'account', shell: AppShell)
final class AccountPage extends StatelessWidget {
  const AccountPage({super.key});

  @override
  Widget build(BuildContext context) => const Scaffold();
}
```

Child paths inherit the nearest parent shell unless they declare their own, so
you do not need to repeat `shell:` on every dashboard page:

```dart
@AppRoute('/dashboard', name: 'dashboard', shell: DashboardShell)
final class DashboardPage extends StatelessWidget {}

@AppRoute('/dashboard/orders', name: 'dashboardOrders')
final class DashboardOrdersPage extends StatelessWidget {}
```

Dust generates the child page as:

```dart
DashboardShell(child: DashboardOrdersPage())
```

Use `shell: AppShell` for layout. Do not create empty shell route marker
classes; if it does not render UI or own navigation state, it should not be a
route.

Use a Flutter `PageTransitionsBuilder` for a route-specific transition:

```dart
@AppRoute(
  '/checkout',
  name: 'checkout',
  transition: BottomToTopPageTransitionsBuilder(),
  fullscreenDialog: true,
  maintainState: true,
)
final class CheckoutPage extends StatelessWidget {
  const CheckoutPage({super.key});

  @override
  Widget build(BuildContext context) => const Scaffold();
}
```

Without `transition`, Dust uses `MaterialPage`. With one, Dust creates a page
route that runs the selected builder at the navigation boundary.

## Observe Route Changes

Override `didChangeRouteStack` on the generated router to record analytics,
breadcrumbs, or debug traces from typed routes:

```dart
@AppRouter(initial: '/', notFound: '/404')
final class RootRouter extends $RootRouter {
  @override
  void didChangeRouteStack(
    RouteStack<AppRoutePath> previous,
    RouteStack<AppRoutePath> next,
  ) {
    analytics.screenView(next.last.location);
  }
}
```

Dust calls this after a route stack is committed. Refreshes and same-location
replacements are ignored so observers do not receive duplicate events.

Use `NavigatorObserver`s when integrating packages that expect Flutter's
standard navigation observer API:

```dart
@AppRouter(initial: '/', notFound: '/404')
final class RootRouter extends $RootRouter {
  RootRouter({required this.analyticsObserver});

  final NavigatorObserver analyticsObserver;

  @override
  List<NavigatorObserver> get observers => [analyticsObserver];
}
```

Handle asynchronous routing failures such as redirect cycles from unawaited
`go()` or `replace()` calls through `onException`:

```dart
@AppRouter(initial: '/', notFound: '/404')
final class RootRouter extends $RootRouter {
  @override
  void onException(Object error, StackTrace stackTrace) {
    errorReporter.capture(error, stackTrace);
  }
}
```

## Diagnostics

Enable runtime logs while debugging parsing, redirects, guards, and stack
changes:

```dart
@override
bool get debugLogDiagnostics => true;
```

The router prints messages through Flutter's `debugPrint` with an `AppRouter:`
prefix. Keep diagnostics disabled in normal use.

## Example

The [shopping app](../../examples/shopping_app) demonstrates public and
protected routes, auth refresh, injected guards, URL round trips, restored deep
link stacks, transitions, and generated navigation.

## Router DX Task List

Dust routing should stay easy to remember: `@AppRouter` defines the app router,
and `@AppRoute` defines pages, shells, branches, guards, and transitions.

Current implementation tasks:

- [x] Keep shells as plain widgets passed through `shell: AppShell`.
- [x] Inherit the nearest parent shell for child paths.
- [x] Validate local shell widgets expose a required named `Widget child`
  constructor parameter.
- [x] Route unawaited navigation failures through `RouterBase.onException`.
- [x] Pass `RouterBase.observers` to the generated root `Navigator`.
- [x] Cover shell imports, constructor mistakes, shell inheritance, and shell
  override behavior in route plugin tests.
- [x] Cover Dart runtime parser, controller, guard-chain, and stack lifecycle
  edge cases.
- [x] Add route diagnostics that print shell, guard, redirect, and branch
  decisions together. Tracked in
  [#405](https://github.com/y3l1n4ung/dust/issues/405).
- [x] Add first-class branch/stateful tab stacks without adding a new
  annotation. Tracked in
  [#406](https://github.com/y3l1n4ung/dust/issues/406).
- [x] Add more web-history tests for back, forward, query, fragment, and
  protected deep-link restore. Tracked in
  [#407](https://github.com/y3l1n4ung/dust/issues/407).

Acceptance tests for branch/stateful-tab work:

- A route using `branch: 'mainTabs'` belongs to an independent tab stack.
- Switching branches preserves each branch stack.
- Browser deep links restore the selected branch and route stack.
- `shell:` remains a layout wrapper; `branch:` is the only tab-stack signal.
- Routes without `branch:` keep today's single-stack Navigator behavior.

Current edge-case test matrix:

| Area | Covered cases |
| --- | --- |
| Shell validation | Local shell widget, imported shell, hidden shell import, missing child, positional child, nullable child, and defaulted child. |
| Shell emission | Inherited parent shell, nearest child shell override, generated page wrapper, and shell metadata consistency. |
| Runtime parser | Platform URI parsing and route-information restoration with query and fragment values. |
| Runtime controller | `RouterController.of`, typed `push`/`pop` result flow, and immutable stack snapshots. |
| Runtime stack | Duplicate route page keys, same-location replace key preservation, pushed-route completion on replace/go, ignored root pop, branch stack preservation, branch deep-link restore, and browser back/forward branch swaps. |
| Diagnostics | Route name, effective shell, branch, redirect target, guard type, guard redirect target, and committed stack logs. |
| Guards | Sync/async guard order, first redirect wins, and sync/async guard exception propagation. |
