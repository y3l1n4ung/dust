# Dust Shopping App

A Flutter commerce showcase for Dust code generation: router, state, HTTP client, serde, and database mapping.

## Features

- FakeStore-backed product catalog, categories, auth, users, and remote cart demo.
- Generated app-level dependency scope with `AppViewModelScope`.
- Product discovery with search, category filtering, and sorting.
- Local wishlist with serde persistence.
- Product detail reviews and recommendations from deterministic fake feature responses.
- Checkout quote preview with fake coupon support (`DUST10`, `SHIPFREE`).
- Order tracking route with fake timeline events.
- Support chat over a local socket-style stream with fake responses so tests stay deterministic.
- Dust DB proof with sqlx-style `@SqlxDatabase`, `@SqlxDao`, `@Query`, and `@Derive([FromRow()])` mapping, flattened rating rows, JSON payloads, try-from decoding, transactions, and offline query metadata.
- Path URL strategy on web, so deep links use clean paths like `/product/7`.

## Run

```bash
cd examples/shopping_app
flutter pub get
cd ../..
cargo run -p dust_cli -- build --root examples/shopping_app --fail-fast
cargo run -p dust_cli -- build --root examples/shopping_app --db --fail-fast
cd examples/shopping_app
flutter run
```

## Verify

```bash
cargo run -p dust_cli -- build --root examples/shopping_app --fail-fast
cargo run -p dust_cli -- build --root examples/shopping_app --db --fail-fast
cd examples/shopping_app
flutter analyze
flutter test
flutter build web
```

## Codegen Contract

- `lib/route.dart` owns `@Router`; `lib/route.g.dart` is generated.
- Pages use `@Route` directly on normal Flutter widgets.
- ViewModels use `@ViewModel` with typed args, for example `AppViewModelArgs(repository, storage)`.
- Data models use `@Derive` for copy/equality/serde output.
- `ShoppingCacheDatabase` uses `@SqlxDatabase`; `ShoppingCacheDao` uses `@SqlxDao` and checked raw SQL `@Query` methods against `migrations/0001_shopping_cache.sql`; run `dust build --db` for SQLite validation and generated DAO output.
- `ShoppingApi` uses Dust HTTP annotations and only declares real FakeStore endpoints.

## API Split

Existing behavior stays live FakeStore by default through `LiveShoppingRepository`.

FakeStore supports:

- `/products`
- `/products/{id}`
- `/products/category/{category}`
- `/products/categories`
- `/carts`
- `/carts/{id}`
- `/carts/user/{userId}`
- `/auth/login`
- `/users/{id}`
- `/users`

Fake local responses support showcase-only features that FakeStore does not provide:

- reviews
- wishlist persistence
- checkout quote and coupons
- order tracking
- support chat

## Main Routes

- `/` products
- `/cart`
- `/checkout`
- `/wishlist`
- `/demo-carts`
- `/orders`
- `/orders/:orderId`
- `/product/:productId`
- `/support/chat`

The Flutter app calls `usePathUrlStrategy()` at startup. Configure static web hosting to serve `index.html` for unknown paths before deploying these deep links outside `flutter run`.
