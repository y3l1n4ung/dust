# Typed Routing

Dust generates a typed Flutter Navigator 2.0 router from annotated widgets.

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
and `@AppRoute` defines pages, shells, guards, transitions, and future route
branching options.

Current implementation tasks:

- [x] Keep shells as plain widgets passed through `shell: AppShell`.
- [x] Inherit the nearest parent shell for child paths.
- [x] Validate local shell widgets expose a required named `Widget child`
  constructor parameter.
- [x] Route unawaited navigation failures through `RouterBase.onException`.
- [x] Pass `RouterBase.observers` to the generated root `Navigator`.
- [ ] Add route diagnostics that print shell, guard, redirect, and branch
  decisions together. Tracked in
  [#405](https://github.com/y3l1n4ung/dust/issues/405).
- [ ] Add first-class branch/stateful tab stacks without adding a new
  annotation. Tracked in
  [#406](https://github.com/y3l1n4ung/dust/issues/406).
- [ ] Add more web-history tests for back, forward, query, fragment, and
  protected deep-link restore. Tracked in
  [#407](https://github.com/y3l1n4ung/dust/issues/407).

Acceptance tests for future branch/stateful-tab work:

- A route using `branch: 'mainTabs'` belongs to an independent tab stack.
- Switching branches preserves each branch stack.
- Browser deep links restore the selected branch and route stack.
- `shell:` remains a layout wrapper; `branch:` is the only tab-stack signal.
- Routes without `branch:` keep today's single-stack Navigator behavior.
