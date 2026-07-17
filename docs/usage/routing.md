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

A shell wraps a route page and must accept a `child` argument:

```dart
@AppRoute('/account', name: 'account', shell: AppShell)
final class AccountPage extends StatelessWidget {
  const AccountPage({super.key});

  @override
  Widget build(BuildContext context) => const Scaffold();
}
```

Child paths inherit the nearest parent shell unless they declare their own.

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
