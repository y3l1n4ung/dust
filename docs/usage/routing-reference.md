# Routing Reference

Use this page after the [routing quick start](./routing.md) when you need the
generated router API details, URL behavior, guards, shells, observers, or CLI
inspection commands. For app-level patterns, see the
[routing recipes](./routing-recipes.md).

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
parameters are encoded as query values. Query parameters can be required,
nullable, or defaulted:

```dart
@AppRoute('/products/:id', name: 'product', guards: [])
final class ProductPage extends StatelessWidget {
  const ProductPage({
    super.key,
    required this.id,
    required this.from,
    this.tab,
    this.preview = false,
    this.tags = const <String>[],
  });

  final int id;
  final DateTime from;
  final String? tab;
  final bool preview;
  final List<String> tags;

  @override
  Widget build(BuildContext context) => Scaffold(body: Text('Product $id'));
}
```

This route can parse and restore
`/products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews&tags=sale&tags=new`.
Default-valued query parameters are omitted when their value matches the
default. Missing or invalid required query values resolve to the configured
not-found route.

Supported URL types:

| Shape | URL spelling |
| --- | --- |
| `String`, `int`, `double`, `bool` | Single path or query value. |
| enum | Query value uses the Dart enum case name, for example `admin`. |
| `DateTime` | Query value uses ISO-8601 text. |
| `Uri` | Query value is encoded once by `Uri`. |
| `List<String>` | Repeated query values, for example `tag=a&tag=b`. |
| `List<int>` | Repeated query values parsed as integers. |

Route widgets need an unnamed generative constructor. Invalid typed path or
query values resolve to the configured not-found route.

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

### RouteInformation Override

Override `parseRouteInformation` when platform links need app-level
normalization before generated route matching. The method uses Flutter's own
`RouteInformation` type, so app code can keep browser history state while
rewriting the incoming URI:

```dart
import 'package:flutter/widgets.dart' show RouteInformation;

@AppRouter(initial: '/', notFound: '/404')
final class RootRouter extends RootRouterBase {
  @override
  RouteInformation parseRouteInformation(RouteInformation information) {
    final uri = information.uri;

    if (uri.host == 'old.example' &&
        uri.pathSegments.length == 2 &&
        uri.pathSegments[0] == 'item') {
      return RouteInformation(
        uri: Uri(pathSegments: ['products', uri.pathSegments[1]]),
        state: information.state,
      );
    }

    if (uri.hasAuthority && uri.host != 'shop.example') {
      return RouteInformation(
        uri: Uri(path: '/404', queryParameters: {'path': uri.toString()}),
        state: information.state,
      );
    }

    if (uri.path.startsWith('/app/')) {
      return RouteInformation(
        uri: uri.replace(path: uri.path.substring('/app'.length)),
        state: information.state,
      );
    }

    return information;
  }
}
```

Common recipes:

| Incoming URL | Override result | Generated typed route |
| --- | --- | --- |
| `https://shop.example/app/products/42?tab=reviews#details` | `/products/42?tab=reviews#details` | `ProductRoute(id: 42, tab: 'reviews')` |
| `https://old.example/item/42` | `/products/42` | `ProductRoute(id: 42)` |
| `https://evil.test/products/42` | `/404?path=https%3A%2F%2Fevil.test%2Fproducts%2F42` | `NotFoundRoute(...)` |

Use this hook for host allow-listing, subdirectory deploy prefixes, legacy URL
migrations, invite-link normalization, and other routing decisions that should
happen before typed route parsing. Use `redirect` for auth and app-state
decisions after a typed route exists.

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
final class RootRouter extends RootRouterBase {
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
final class RootRouter extends RootRouterBase {
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
final class RootRouter extends RootRouterBase {
  @override
  void onException(Object error, StackTrace stackTrace) {
    errorReporter.capture(error, stackTrace);
  }
}
```

## Route Table

Run `dust route table` when you want a quick reviewable map of the generated
router without opening generated internals:

```bash
dust route table
```

The command is read-only. It prints each route's name, path, page type,
effective shell, effective branch, guards, auth state, and result type:

```text
route table  scanned: 53  routes: 16  time: 139ms
name | path | page | shell | branch | guards | auth | result
--- | --- | --- | --- | --- | --- | --- | ---
products | / | ProductsScreen | - | - | - | public | void
admin | /admin | AdminDashboardScreen | - | - | AdminGuard | protected | void
cart | /cart | CartScreen | - | - | - | public | void
checkout | /checkout | CheckoutScreen | - | - | - | protected | void
supportChat | /support/chat | SupportChatScreen | - | - | - | public | bool
```

The rows above are trimmed from the `examples/shopping_app` output.

`auth` reports the generated `requiresAuth` value. A route is `public` only when
it declares `guards: []` or when it is the router's not-found route; every other
route is `protected`. Read this column instead of `guards`: `cart` and
`checkout` both show `-` guards above, but only `cart` is public.

Use `--root` from monorepos or scripts:

```bash
dust route table --root examples/shopping_app
```

## Route Graph

Run `dust route graph` when you want parent-child route relationships in a
stable Markdown table:

```bash
dust route graph
```

The command is read-only. It prints each route path, nearest parent route path,
route name, page type, effective shell, effective branch, and direct guards:

```text
route graph  scanned: 53  routes: 16  time: 206ms
path | parent | name | page | shell | branch | guards
--- | --- | --- | --- | --- | --- | ---
/ | - | products | ProductsScreen | - | - | -
/admin | / | admin | AdminDashboardScreen | - | - | AdminGuard
/products/:id | / | productDetail | ProductDetailScreen | - | - | -
/support/chat | / | supportChat | SupportChatScreen | - | - | -
```

Use `--root` from monorepos or scripts:

```bash
dust route graph --root examples/shopping_app
```

## Route Fixtures

Run `dust route fixtures` when you want copyable deep-link QA examples without
deriving every path and query shape by hand:

```bash
dust route fixtures
```

The command is read-only. It prints valid and invalid examples for path URLs,
web URLs, app-link style URLs, custom-scheme URLs, typed path params, typed
query params, preserved fragments, and not-found routes:

```text
route fixtures  scanned: 53  fixtures: 62  time: 206ms
route | case | valid | shape | uri | expected
--- | --- | --- | --- | --- | ---
productDetail | path | true | path | /product/SKU-1001?tab=reviews | typed-route
productDetail | web-url | true | web-url | https://shop.example/product/SKU-1001?tab=reviews | normalize-then-typed-route
productDetail | app-link | true | app-link | https://shop.example/app/product/SKU-1001?tab=reviews | normalize-prefix-then-typed-route
productDetail | custom-scheme | true | custom-scheme | shopping:///product/SKU-1001?tab=reviews | normalize-scheme-then-typed-route
productDetail | fragment-preserved | true | path | /product/SKU-1001?tab=reviews#details | typed-route-preserve-fragment
- | not-found | false | path | /__dust_missing_route__ | not-found-route
```

Fixtures use the documented normalization shapes from this guide:
`https://shop.example/...`, `https://shop.example/app/...`, and
`shopping:///...`. Apps with different hosts, prefixes, or schemes should keep
their `parseRouteInformation` override covered by app-level tests.

Fragments are not decoded into typed route constructor fields. Dart's `Uri`
normalizes malformed percent escapes before Dust sees the route information, so
fragment fixture rows focus on preservation and round-trip behavior. Invalid
fixture rows focus on typed path/query parsing and not-found routing.

Use `--root` from monorepos or scripts:

```bash
dust route fixtures --root examples/shopping_app
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
