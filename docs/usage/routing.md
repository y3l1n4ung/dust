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

Create one router entrypoint at `lib/route.dart`:

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

Generate the router and hand its configuration to Flutter:

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
> Keep `route.dart` as the app-facing routing entrypoint. It imports and exports
> generated files under `route/`; route pages import `route.dart` to use
> annotations and generated navigation helpers.

Exactly one `@AppRouter` is allowed per project. A second one fails generation
with `exactly one @AppRouter is allowed in a Dust route workspace`. Nested
layouts and independent tab histories are handled by shells and branches
instead, so they never need a second router.

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

## Deep Links and Browser URLs

The generated parser converts incoming platform and browser URIs into typed
routes, so `/products/42?tab=reviews` becomes
`ShopProductRoute(id: 42, tab: 'reviews')`. Dust also rebuilds the stack from
matching path prefixes: `/orders/ORDER-9` restores the initial page, `/orders`,
and the order detail page.

Unknown query values and URI fragments are preserved in `route.location`, so
auth redirects round-trip campaign parameters and anchors the page does not
model.

> [!TIP]
> Test both `parseShopRoute(Uri.parse(url))` and the generated route's
> `location`. That catches decoding and round-trip regressions before testing a
> full platform deep-link flow.

Override `parseRouteInformation` when platform links need app-level
normalization before route matching:

```dart
import 'package:flutter/widgets.dart' show RouteInformation;

@AppRouter(initial: '/', notFound: '/404')
final class ShopRouter extends $ShopRouter {
  @override
  RouteInformation parseRouteInformation(RouteInformation information) {
    final uri = information.uri;

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

Use this hook for host allow-listing, subdirectory deploy prefixes, and legacy
URL migrations. Use `redirect` for decisions that depend on app state.

## Redirects and Authentication

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

## Route Guards

Use guards for route-specific access checks:

```dart
final class AdminGuard implements RouteGuard<ShopRoutePath> {
  const AdminGuard(this.auth);

  final AuthViewModel auth;

  @override
  ShopRoutePath? canActivate(ShopRoutePath route) {
    return auth.state.isAdmin ? null : const ShopHomeRoute();
  }
}

@AppRoute('/admin', name: 'admin', guards: [AdminGuard])
final class AdminPage extends StatelessWidget {
  const AdminPage({super.key});
}
```

A guard returns `null` to allow navigation or another route to redirect.
Implement `AsyncRouteGuard<ShopRoutePath>` when the decision needs a `Future`.
Mixed sync and async guards run in annotation order, and the first redirect
wins.

Every class in `guards:` must implement one of those two contracts. Generated
guard lists are typed `List<RouteGuardBase<ShopRoutePath>>`, so a class that
implements neither is an analyzer error rather than a guard that never runs.

Guard constructor dependencies are matched to router fields by type. Dust passes
`ShopRouter.auth` to `AdminGuard` above. Generation fails when a required
dependency is missing or ambiguous.

> [!NOTE]
> Route guards control navigation, not backend authorization. Enforce access on
> the server as well.

## Shells and Branches

A shell is a normal widget that takes a required named `Widget child`. There is
no separate shell annotation:

```dart
final class AppShell extends StatelessWidget {
  const AppShell({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) => Scaffold(body: child);
}

@AppRoute('/dashboard', name: 'dashboard', shell: AppShell)
final class DashboardPage extends StatelessWidget {
  const DashboardPage({super.key});
}

@AppRoute('/dashboard/orders', name: 'dashboardOrders')
final class DashboardOrdersPage extends StatelessWidget {
  const DashboardOrdersPage({super.key});
}
```

Child paths inherit the nearest parent shell, so the child above is generated as
`AppShell(child: DashboardOrdersPage())`. Do not create empty shell marker
routes; if it does not render UI or own navigation state, it should not be a
route.

Use `branch:` when routes need independent tab stacks. Keep `shell:` for layout
and `branch:` for navigation state:

```dart
@AppRoute('/tabs/home', name: 'tabHome', shell: AppShell, branch: 'mainTabs')
final class TabHomePage extends StatelessWidget {
  const TabHomePage({super.key});
}

@AppRoute('/tabs/home/details', name: 'tabHomeDetails')
final class TabHomeDetailsPage extends StatelessWidget {
  const TabHomeDetailsPage({super.key});
}
```

Switching to another branch and back restores the first branch's stack, so
`/tabs/home` reopens with `/tabs/home/details` still on top. Child paths inherit
the nearest parent `branch:` just as they inherit `shell:`, and Dust generates a
stable constant per branch value, such as `shopBranchMainTabs`, reused in route
metadata and debug helpers.

Without `transition`, Dust uses `MaterialPage`. With one, it creates a page route
that runs the selected `PageTransitionsBuilder` at the navigation boundary.

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
> Use route results for one-time answers such as pickers and confirmations. Keep
> shareable state in path or query parameters.

## Observe Route Changes

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

Dust calls this after a stack is committed. Refreshes and same-location
replacements are skipped, so observers do not see duplicate events.

Provide `observers` for packages that expect Flutter's `NavigatorObserver` API,
and `onException` for asynchronous routing failures such as redirect cycles from
unawaited `go()` or `replace()` calls:

```dart
@AppRouter(initial: '/', notFound: '/404')
final class ShopRouter extends $ShopRouter {
  ShopRouter({required this.analyticsObserver});

  final NavigatorObserver analyticsObserver;

  @override
  List<NavigatorObserver> get observers => [analyticsObserver];

  @override
  void onException(Object error, StackTrace stackTrace) {
    errorReporter.capture(error, stackTrace);
  }
}
```

## Web URLs

Flutter web uses hash URLs such as `/#/products/42` by default. For normal path
URLs, add the Flutter SDK web plugins dependency and call `usePathUrlStrategy()`
before `runApp`:

```dart
import 'package:flutter_web_plugins/url_strategy.dart';

void main() {
  usePathUrlStrategy();

  final router = ShopRouter();
  runApp(MaterialApp.router(routerConfig: router.config));
}
```

Path URLs also need the web host to serve `index.html` for unknown app paths,
otherwise a browser refresh on `/products/42` never reaches the Flutter app.
Dust cannot do this from generated Dart. Most hosts support it directly, for
example NGINX `try_files $uri $uri/ /index.html;`; see Flutter's
[URL strategies guide](https://docs.flutter.dev/ui/navigation/url-strategies).

For a subdirectory deploy, build with a matching base href and strip the prefix
in `parseRouteInformation` as shown above:

```bash
flutter build web --base-href /app/
```

## Diagnostics

Enable runtime logs while debugging parsing, redirects, guards, and stack
changes:

```dart
@override
bool get debugLogDiagnostics => true;
```

The router prints through Flutter's `debugPrint` with an `AppRouter:` prefix.
Keep diagnostics disabled in normal use.

## Example

The [shopping app](../../examples/shopping_app) demonstrates public and
protected routes, auth refresh, injected guards, URL round trips, restored deep
link stacks, transitions, and generated navigation.
