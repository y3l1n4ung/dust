# Router Web URL Deployment

Dust routers use Flutter's Navigator 2.0 APIs. Dust can parse and restore typed
routes, but the web host still decides whether a direct browser request reaches
the Flutter app.

Use this guide when a route works after in-app navigation but fails after a
browser refresh or when opening a shared URL directly.

## Choose A URL Strategy

| Strategy | Example URL | Host requirement | Use when |
| :--- | :--- | :--- | :--- |
| Hash URLs | `https://shop.example/#/products/42` | Serve `index.html` at `/`. The route after `#` stays in the browser. | You need the simplest deploy, especially on hosts without rewrite support. |
| Path URLs | `https://shop.example/products/42` | Rewrite unknown app paths to `index.html`. | You need shareable, normal URLs and control the web host. |
| Subdirectory path URLs | `https://shop.example/app/products/42` | Build with base href `/app/` and rewrite unknown `/app/**` paths to the app's `index.html`. | The Flutter app is not hosted at the domain root. |

Dust sees the same typed route after Flutter hands it the app URI:

| Browser URL | Generated typed route |
| :--- | :--- |
| `/#/products/42?tab=reviews` | `ProductRoute(id: 42, tab: 'reviews')` |
| `/products/42?tab=reviews#details` | `ProductRoute(id: 42, tab: 'reviews')` |
| `/app/products/42?tab=reviews#details` after prefix normalization | `ProductRoute(id: 42, tab: 'reviews')` |
| `/products/not-an-int?from=email` | configured not-found route |

Unknown query values and URI fragments are preserved in `route.location`, so
redirects can round-trip campaign tags, anchors, and other unmodeled URL data.

## Path URLs

Flutter web uses hash URLs by default. For path URLs, add the Flutter SDK web
plugins dependency and call `usePathUrlStrategy()` before `runApp`:

```yaml
dependencies:
  flutter:
    sdk: flutter
  flutter_web_plugins:
    sdk: flutter
```

```dart
import 'package:flutter_web_plugins/url_strategy.dart';

void main() {
  usePathUrlStrategy();

  final router = RootRouter();
  runApp(MaterialApp.router(routerConfig: router.config));
}
```

Then configure the host so requests for unknown app paths serve `index.html`
instead of a host-level 404 page. Dust cannot do this from generated Dart code;
the browser must receive the Flutter app first.

## Subdirectory Deploys

When the app is hosted below a path prefix such as `/app/`, build the Flutter
web app with a matching base href:

```bash
flutter build web --base-href /app/
```

The host must serve the built files from that same prefix. Direct requests such
as `/app/products/42` must return the app's `index.html`.

Normalize the prefix before generated route parsing:

```dart
import 'package:flutter/widgets.dart' show RouteInformation;

@AppRouter(initial: '/', notFound: '/404')
final class RootRouter extends RootRouterBase {
  @override
  RouteInformation parseRouteInformation(RouteInformation information) {
    final uri = information.uri;
    if (uri.path == '/app') {
      return RouteInformation(
        uri: uri.replace(path: '/'),
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

For `https://shop.example/app/products/42?tab=reviews#details`, the generated
parser receives `/products/42?tab=reviews#details`.

## Host Rewrites

These examples assume `flutter build web` writes to `build/web`.

| Host | Path URL support | Configuration |
| :--- | :--- | :--- |
| NGINX | Full support | `try_files $uri $uri/ /index.html;` inside the app `location`. |
| Firebase Hosting | Full support | Rewrite all unmatched app paths to `/index.html`. |
| Vercel | Full support | Add a `vercel.json` rewrite from all app paths to `/index.html`. |
| Cloudflare Pages | Full support by default for SPA projects | Do not add a top-level `404.html` if you want Pages' default SPA fallback. |
| GitHub Pages | Prefer hash URLs | GitHub Pages has no normal rewrite config. A copied `404.html` fallback can load the app, but the first response is still a 404. |

### NGINX

Root deploy:

```nginx
server {
    listen 80;
    server_name shop.example;
    root /var/www/shopping_app/build/web;

    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

Subdirectory deploy:

```nginx
location /app/ {
    alias /var/www/shopping_app/build/web/;
    try_files $uri $uri/ /app/index.html;
}
```

### Firebase Hosting

```json
{
  "hosting": {
    "public": "build/web",
    "ignore": ["firebase.json", "**/.*", "**/node_modules/**"],
    "rewrites": [
      {
        "source": "**",
        "destination": "/index.html"
      }
    ]
  }
}
```

For a subdirectory deploy, place the built app under that directory and rewrite
only that prefix:

```json
{
  "hosting": {
    "public": "public",
    "rewrites": [
      {
        "source": "/app/**",
        "destination": "/app/index.html"
      }
    ]
  }
}
```

### Vercel

```json
{
  "rewrites": [
    {
      "source": "/(.*)",
      "destination": "/index.html"
    }
  ]
}
```

If the same project also serves APIs or static files with their own routes,
place more specific rules before the catch-all.

### Cloudflare Pages

Cloudflare Pages treats a project without a top-level `404.html` as a
single-page app and serves the root app for unknown paths. For a Dust Flutter
web app using path URLs, avoid shipping a top-level `404.html` unless you have a
separate fallback strategy.

### GitHub Pages

GitHub Pages is a poor fit for root path URLs because it does not provide a
normal SPA rewrite rule. Use hash URLs for predictable behavior:

```text
https://yelinaung.github.io/shopping_app/#/products/42
```

If you must use path-looking URLs, the common workaround is copying
`build/web/index.html` to `build/web/404.html` before publishing. That lets the
Flutter app load after GitHub Pages serves the 404 page, but the HTTP response
status remains 404.

## Verification Checklist

Run these checks against the deployed URL, not only `flutter run -d chrome`:

| Check | Expected result |
| :--- | :--- |
| Open `/products/42` directly | App loads and renders `ProductRoute(id: 42)`. |
| Refresh `/products/42?tab=reviews#details` | App stays on the product route and preserves the query and fragment in `route.location`. |
| Navigate in app to `/cart`, then browser Back | Router restores the previous typed route. |
| Browser Forward after Back | Router restores `/cart` without creating a duplicate stack. |
| Open an invalid typed URL such as `/products/not-an-int` | Generated parser resolves the configured not-found route. |
| Open an unsafe host or legacy URL shape | `parseRouteInformation` normalizes it or sends it to not-found before typed parsing. |

Also keep unit coverage around generated parsing:

```dart
test('product deep link round-trips', () {
  final route = parseAppRoute(
    Uri.parse('/products/42?tab=reviews&utm=email#details'),
  ) as ProductRoute;

  expect(route.id, 42);
  expect(route.tab, 'reviews');
  expect(route.location, '/products/42?tab=reviews&utm=email#details');
});
```

## References

- Flutter web URL strategies: <https://docs.flutter.dev/ui/navigation/url-strategies>
- NGINX `try_files`: <https://nginx.org/r/try_files>
- Firebase Hosting rewrites: <https://firebase.google.com/docs/hosting/full-config>
- Vercel rewrites: <https://vercel.com/docs/routing/rewrites>
- Cloudflare Pages SPA rendering: <https://developers.cloudflare.com/pages/configuration/serving-pages/#single-page-application-spa-rendering>
- GitHub Pages SPA fallback discussion: <https://github.com/orgs/community/discussions/27676>
