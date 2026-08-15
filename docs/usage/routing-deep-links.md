# Routing Deep Links and Web URLs

Dust route data is URL-first. The generated parser turns browser, app-link, and
platform URIs into typed route objects, and generated route objects restore back
to URL text through `location`.

## Round Trips

`/products/42?tab=reviews` parses to
`ShopProductRoute(id: 42, tab: 'reviews')`. Dust also rebuilds the stack from
matching path prefixes: `/orders/ORDER-9` restores the initial page, `/orders`,
and the order detail page.

Unknown query values and URI fragments are preserved in `route.location`, so
auth redirects round-trip campaign parameters and anchors the page does not
model.

> [!TIP]
> Test both parse and restore behavior. That catches decoding and round-trip
> regressions before testing a full platform deep-link flow.

```dart
final route = parseShopRoute(Uri.parse('/products/42?tab=reviews#details'));

expect(route, isA<ShopProductRoute>());
expect(route.location, '/products/42?tab=reviews#details');
```

## Normalize Incoming Links

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

## Safe Redirect Locations

Redirect routes often store the blocked destination:

```dart
return ShopLoginRoute(redirectPath: route.location);
```

> [!IMPORTANT]
> Validate that value before reopening it. Reject external hosts and unknown
> routes instead of treating arbitrary input as an internal destination.

## Flutter Web URLs

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

> [!IMPORTANT]
> Path URLs also need the web host to serve `index.html` for unknown app paths,
> otherwise a browser refresh on `/products/42` never reaches the Flutter app.
> Dust cannot do this from generated Dart.

Most hosts support this directly, for example NGINX
`try_files $uri $uri/ /index.html;`; see Flutter's
[URL strategies guide](https://docs.flutter.dev/ui/navigation/url-strategies).

For a subdirectory deploy, build with a matching base href and strip the prefix
in `parseRouteInformation`:

```bash
flutter build web --base-href /app/
```
