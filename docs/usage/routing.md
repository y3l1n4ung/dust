# Typed Routing

Dust generates a typed Flutter Navigator 2.0 router from annotated widgets.
For a source-grounded comparison with go_router, AutoRoute, Beamer, and
hand-written Router 2.0, see the
[router DX comparison](./routing-dx-comparison.md).

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
final class RootRouter extends RootRouterBase {}
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
> generated files under `route/`; route pages import `route.dart` to use
> annotations and generated navigation helpers.

## Shopping Cart Entrypoint

The [shopping app](../../examples/shopping_app) keeps the full routing surface
behind one handwritten file: `lib/route.dart`.

```dart
import 'package:dust_flutter/route.dart';

import 'features/auth/models/auth_state.dart';
import 'features/auth/view_models/auth_view_model.dart';
import 'route/routes.g.dart';

export 'package:dust_flutter/route.dart';
export 'route/routes.g.dart';

@AppRouter(initial: '/', notFound: '/404')
final class ShoppingRouter extends ShoppingRouterBase {
  ShoppingRouter({required this.auth});

  final AuthViewModel auth;

  @override
  ShoppingRoutePath? redirect(ShoppingRoutePath route) {
    final status = auth.state.status;
    if (status == AuthStatus.loading || status == AuthStatus.initial) {
      return null;
    }
    if (!auth.state.isAuthenticated && route.requiresAuth) {
      return ShoppingLoginRoute(redirectPath: route.location);
    }
    return null;
  }
}
```

Route pages import the same entrypoint. They do not import generated files:

```dart
import 'package:flutter/material.dart';
import 'package:shopping_app/route.dart';

@AppRoute('/cart', name: 'cart', guards: [])
final class CartScreen extends StatelessWidget {
  const CartScreen({super.key});
}

@AppRoute(
  '/checkout',
  name: 'checkout',
  transition: BottomToTopPageTransitionsBuilder(),
  fullscreenDialog: true,
)
final class CheckoutScreen extends StatelessWidget {
  const CheckoutScreen({super.key});
}

@AppRoute(
  '/order-confirmation/:orderId',
  name: 'orderConfirmation',
  guards: [],
  transition: ZoomPageTransitionsBuilder(),
)
final class OrderConfirmationScreen extends StatelessWidget {
  const OrderConfirmationScreen({required this.orderId, super.key});

  final String orderId;
}
```

App code also imports only `route.dart`:

```dart
import 'package:shopping_app/route.dart';

final router = ShoppingRouter(auth: authViewModel);

MaterialApp.router(routerConfig: router.config);

context.navigator.cart().go();
context.navigator.checkout().push();
context.navigator
    .orderConfirmation(orderId: 'ORDER 1/2')
    .replace();
```

Call chain for a signed-out checkout deep link:

```text
browser opens /checkout
parseShoppingRoute(Uri.parse('/checkout'))
ShoppingCheckoutRoute()
ShoppingRouter.redirect(ShoppingCheckoutRoute())
ShoppingLoginRoute(redirectPath: /checkout)
Navigator.pages = [/, /login?redirectPath=%2Fcheckout]
```

Generated files stay under `lib/route/`. The only generated file that
`route.dart` names directly is `route/routes.g.dart`, which is a generated
barrel for route paths, navigation helpers, metadata, and runtime glue.

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

## Next Steps

| Need | Guide |
| --- | --- |
| Common app patterns | [Routing recipes](./routing-recipes.md) |
| API details and diagnostics | [Routing reference](./routing-reference.md) |
| Flutter web path URLs and server rewrites | [Router web URL deployment](./routing-web-deployment.md) |
| Source-grounded router comparison | [Router DX comparison](./routing-dx-comparison.md) |

## Example

The [shopping app](../../examples/shopping_app) demonstrates public and
protected routes, auth refresh, injected guards, URL round trips, restored deep
link stacks, transitions, and generated navigation.
