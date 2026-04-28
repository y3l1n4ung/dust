# Route Annotation Plan

## Goal

Generate typed Flutter routing on top of Navigator 2.0. Dust should create route
configuration, typed route data, URL parsing, and navigation helpers from
annotations.

## Package Shape

- Dart annotation package: `route_annotation`
- Rust plugin crate: `dust_plugin_route`
- Optional Flutter runtime package: `dust_router`

## API Sketch

```dart
@DustRoutes([
  DustRoute(path: '/', page: HomePage),
  DustRoute(path: '/products/:sku', page: ProductPage),
  DustRoute(path: '/settings/profile', page: ProfilePage),
])
final class AppRoutes {}
```

Generated output:

```dart
final routerConfig = _$AppRoutesRouter.config();

context.goProduct(sku: 'sku-1');
final location = ProductRoute(sku: 'sku-1').location;
```

## Navigator 2.0 Scope

- Generate `RouteInformationParser`.
- Generate `RouterDelegate`.
- Generate typed route data classes.
- Generate path and query parsing.
- Generate nested route support.
- Generate redirect and guard hooks.
- Keep Flutter dependency out of Dust core crates.

## Tests

- Golden tests for static, dynamic, query, and nested routes.
- Dart analyzer tests in a Flutter example package.
- Runtime widget tests for deep links and back navigation.
- Negative tests for duplicate paths and missing path params.

## Done

- A Flutter app can use Dust-generated `routerConfig`.
- URLs round-trip through typed route data.
- Navigator 2.0 behavior is test-covered.
