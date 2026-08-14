# Shopping App

End-to-end Flutter example for Dust router, state, HTTP, JSON, i18n, and
Database generation.

This app is the main regression surface for using Dust in a real Flutter app
instead of isolated model fixtures.

## What It Shows

| Area | Where to look |
| :--- | :--- |
| Routing and guards | [`lib/route.dart`](lib/route.dart), [`lib/route/routes.g.dart`](lib/route/routes.g.dart) |
| ViewModel scopes | [`lib/main.dart`](lib/main.dart), [`lib/core/view_models/app_view_model.dart`](lib/core/view_models/app_view_model.dart) |
| HTTP client | [`lib/core/api/shopping_api.dart`](lib/core/api/shopping_api.dart) |
| JSON models | [`lib/features/products/models/product.dart`](lib/features/products/models/product.dart), [`lib/features/cart/models/cart_state.dart`](lib/features/cart/models/cart_state.dart) |
| i18n | [`dust.yaml`](dust.yaml), [`assets/i18n/en/shop.arb`](assets/i18n/en/shop.arb), [`assets/i18n/my/shop.arb`](assets/i18n/my/shop.arb) |
| Database | [`lib/core/db/shopping_cache_database.dart`](lib/core/db/shopping_cache_database.dart), [`migrations/0001_shopping_cache.sql`](migrations/0001_shopping_cache.sql) |

The app uses live FakeStore endpoints for core catalog/auth/cart data and local
fake responses for showcase-only flows such as checkout quotes, order tracking,
wishlist persistence, and support chat.

## Run It

From the repository root:

```bash
cargo run -q -p dust_cli -- build --root examples/shopping_app --fail-fast
cargo run -q -p dust_cli -- db build --root examples/shopping_app --fail-fast
cd examples/shopping_app
flutter pub get
flutter run
```

## Validate

```bash
cargo run -q -p dust_cli -- check --root examples/shopping_app --fail-fast
cargo run -q -p dust_cli -- db build --root examples/shopping_app --fail-fast
cd examples/shopping_app
flutter analyze
flutter test
flutter build web
```

## Generated Surfaces

- `@AppRouter` generates typed route classes and parser helpers in
  generated files under [`lib/route/`](lib/route/).
- [`lib/route.dart`](lib/route.dart) is the single app-facing routing
  entrypoint. App code imports it instead of generated files directly.
- `@AppRoute` on Flutter pages declares paths, route parameters, and guards.
- `@ViewModel` generates scopes, typed args, readers, and watchers.
- `@Derive` generates copy, equality, JSON, validation, and row-mapping helpers
  for app models.
- `@HttpClient` generates a Dio-backed FakeStore client.
- `@SqlxDatabase`, `@SqlxDao`, and `@Query` generate checked SQLite access for
  the shopping cache.
- Dust i18n generates [`lib/i18n/app_i18n.g.dart`](lib/i18n/app_i18n.g.dart)
  from the configured ARB locales.

## Main Routes

- `/`
- `/cart`
- `/checkout`
- `/order-confirmation/:orderId`
- `/wishlist`
- `/demo-carts`
- `/orders`
- `/orders/:orderId`
- `/product/:productId`
- `/support/chat`

The app calls `usePathUrlStrategy()` at startup. For deployed web builds,
configure the host to serve `index.html` for unknown paths so direct deep links
such as `/product/7` load the Flutter app. Host examples are covered in the
[routing guide](../../docs/usage/routing.md#web-urls).

## Route Entrypoint

Handwritten app code only needs `lib/route.dart`:

```dart
import 'package:dust_flutter/route.dart';

import 'features/auth/view_models/auth_view_model.dart';
import 'route/routes.g.dart';

export 'package:dust_flutter/route.dart';
export 'route/routes.g.dart';

@AppRouter(initial: '/', notFound: '/404')
final class ShoppingRouter extends ShoppingRouterBase {
  ShoppingRouter({required this.auth});

  final AuthViewModel auth;
}
```

Every route page and caller imports the same entrypoint:

```dart
import 'package:shopping_app/route.dart';

context.navigator.cart().go();
context.navigator.checkout().push();
context.navigator.orderConfirmation(orderId: 'ORDER-42').replace();
```

The generated `lib/route/routes.g.dart` file is the barrel for generated route
paths, navigation helpers, metadata, and runtime glue. User code does not
re-export those generated pieces one by one.

## More Docs

- [Root README](../../README.md)
- [Usage guide](../../docs/usage/README.md)
- [Routing guide](../../docs/usage/routing.md)
- [State guide](../../docs/usage/state.md)
- [HTTP guide](../../docs/usage/http.md)
- [i18n guide](../../docs/usage/i18n.md)
- [Database guide](../../docs/usage/db.md)
