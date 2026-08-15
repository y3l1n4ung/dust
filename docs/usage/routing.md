# Typed Routing

Dust generates a typed Flutter Navigator 2.0 router from annotated widgets.
One `@AppRouter` class and one `@AppRoute` per page produce the parser, the
delegate, guards, shells, and typed navigation helpers.

## Add the Package

Install the Dust CLI from the [root guide](../../README.md#installation), then
add the Flutter runtime:

```bash
flutter pub add dust_flutter
```

## Quick Start

Create one app-facing router entrypoint at `lib/route.dart`:

```dart
import 'package:dust_flutter/route.dart';

import 'route/routes.g.dart';

export 'package:dust_flutter/route.dart';
export 'route/routes.g.dart';

@AppRouter(initial: '/', notFound: '/404')
final class ShopRouter extends $ShopRouter {}
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

Generate the router and pass its config to Flutter:

```bash
dust build
```

```dart
void main() {
  final router = ShopRouter();
  runApp(MaterialApp.router(routerConfig: router.config));
}
```

> [!IMPORTANT]
> Keep `route.dart` as the single app-facing routing entrypoint. It imports and
> exports generated files under `route/`; route pages import `route.dart` to use
> annotations and generated navigation helpers.

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

If `name` is omitted, Dust derives it from the widget class name by stripping a
trailing `Page`, `Screen`, or `View`. `ProductDetailsScreen` becomes
`productDetails()` and `ShopProductDetailsRoute`.

System back dismisses dialogs, modal sheets, and other imperatively pushed
routes before popping a generated page, and honours `PopScope`.

## Route Options

```dart
@AppRoute(
  '/checkout',
  name: 'checkout',
  result: bool,
  shell: AppShell,
  branch: 'mainTabs',
  guards: [CartGuard],
  transition: BottomToTopPageTransitionsBuilder(),
  fullscreenDialog: true,
  maintainState: true,
)
```

| Option | Default | Purpose |
| :--- | :--- | :--- |
| `name` | derived from the class name | Route and navigation helper name. |
| `result` | `void` | Type returned by `push()` when the route is popped. |
| `shell` | inherited from the nearest parent path | Layout widget wrapping the page. |
| `branch` | inherited from the nearest parent path | Stateful tab stack the route belongs to. |
| `guards` | protected | Route-specific access checks. `guards: []` marks a route public. |
| `transition` | `MaterialPage` | `PageTransitionsBuilder` used at the page boundary. |
| `fullscreenDialog` | `false` | Presents the page as a fullscreen dialog. |
| `maintainState` | `true` | Keeps page state alive while inactive. |

Routes are protected unless they declare `guards: []`. The router's not-found
route is always public.

For shell inheritance and independent branch stacks, see
[Shells and Branches](./routing-shells.md). For guard injection and access
checks, see [Guards](./routing-guards.md).

## Route Parameters

Path parameters match required, non-nullable constructor parameters. Other
parameters become query values, and may be required, nullable, or defaulted:

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
}
```

This route parses and restores
`/products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews&tags=sale&tags=new`.
Query parameters equal to their default are omitted from the generated URL.
Missing or invalid required values resolve to the configured not-found route.

Supported URL types:

| Shape | URL spelling |
| --- | --- |
| `String`, `int`, `double`, `bool` | Single path or query value. |
| enum | Query value uses the Dart enum case name, for example `admin`. |
| `DateTime` | Query value uses ISO-8601 text. |
| `Uri` | Query value is encoded once by `Uri`. |
| `List<String>` | Repeated query values, for example `tags=a&tags=b`. |
| `List<int>` | Repeated query values parsed as integers. |

Route widgets need an unnamed generative constructor. Anything richer than a URL
primitive should be loaded from app state after navigation.

## Redirects

Use the router's `redirect` method for app-wide auth decisions:

```dart
@AppRouter(initial: '/', notFound: '/404')
final class ShopRouter extends $ShopRouter {
  ShopRouter({required this.auth});

  final AuthViewModel auth;

  @override
  ShopRoutePath? redirect(ShopRoutePath route) {
    if (!auth.state.isAuthenticated && route.requiresAuth) {
      return ShopLoginRoute(redirectPath: route.location);
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
> Validate saved redirect locations before reopening them. Reject external hosts
> and unknown routes instead of treating arbitrary input as an internal
> destination.

## Deep Links and Web URLs

The generated parser converts incoming platform and browser URIs into typed
routes, so `/products/42?tab=reviews` becomes
`ShopProductRoute(id: 42, tab: 'reviews')`.

For app-link normalization, host allow-listing, subdirectory deploys, and
Flutter web path URLs, see [Deep Links and Web URLs](./routing-deep-links.md).

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

## Observability

Override `didChangeRouteStack` to record analytics or breadcrumbs from typed
routes:

```dart
@AppRouter(initial: '/', notFound: '/404')
final class ShopRouter extends $ShopRouter {
  @override
  void didChangeRouteStack(
    RouteStack<ShopRoutePath> previous,
    RouteStack<ShopRoutePath> next,
  ) {
    analytics.screenView(next.last.location);
  }
}
```

Provide `observers` for packages that expect Flutter's `NavigatorObserver` API,
and `onException` for asynchronous routing failures such as redirect cycles from
unawaited `go()` or `replace()` calls.

Enable runtime logs while debugging parsing, redirects, guards, and stack
changes:

```dart
@override
bool get debugLogDiagnostics => true;
```

## Example

The [shopping app](../../examples/shopping_app) demonstrates public and
protected routes, auth refresh, injected guards, URL round trips, restored deep
link stacks, transitions, and generated navigation.
