# Serving a web application

One process, one port: the API and the built application it serves. Flutter
web, React, Vue, Svelte, or a hand-written bundle — anything that ships a
document plus assets and routes in the browser.

```dart
final app = Router()
  ..nest('/api', apiRoutes)
  ..fallback(staticFiles('build/web', html: true));

await serveRouter(app, InternetAddress.anyIPv4, 8080);
```

`fallback` is the right place for it. API routes win, and everything they do
not claim is a page view.

## What HTML mode adds

The name follows Starlette's `StaticFiles(directory=..., html=True)`: it is
still static file serving, with the behaviour a served application needs.

### Client-side routes reach the application

`/orders/42` exists only inside the Flutter router. On disk there is no such
file, so plain static serving answers 404 and a shared link is broken. This
answers `index.html` instead, and the browser keeps the URL it asked for.

Only `GET` and `HEAD`. A `POST` to a path no API route matched is a wrong
request, and answering it with an HTML document would hide that behind a 200.

### The shell is revalidated; everything else is not

This is the failure that shows up a day after a deploy: users are on the old
build, and a hard refresh fixes it. It happens because some of the files a
build regenerates carry no content hash, so a cache that keeps them keeps the
whole application. Flutter is the worst case — its entry files are all
unhashed — but every toolchain has at least the document.

| Files | `cache-control` |
| :--- | :--- |
| `defaultRevalidatedFiles` — the document, service workers, and the unhashed entry files the common toolchains emit | `no-cache` |
| `assets/`, `canvaskit/`, fonts, everything else | `public, max-age=31536000, immutable` |

`no-cache` still allows a 304, so a repeat visit costs one conditional request
rather than a download. Shorten the other side with `immutableFor:`.

A bundler that hashes every asset needs only the document, so narrow the set:

```dart
staticFiles('dist', html: true, revalidate: const {'index.html'})
```

A document under another name is `defaultDocument:`:

```dart
staticFiles(
  'dist',
  html: true,
  defaultDocument: 'app.html',
  revalidate: const {'app.html'},
)
```

A directory request counts as the document it resolved to, so `/` and
`/admin/` are treated as `index.html` rather than falling into the immutable
bucket.

### Cross-origin isolation

```dart
staticFiles('build/web', html: true, crossOriginIsolated: true)
```

An application that needs `SharedArrayBuffer` — a Flutter `--wasm` build, or
anything using threads — only gets it in a cross-origin-isolated document. That adds:

```
cross-origin-opener-policy: same-origin
cross-origin-embedder-policy: require-corp
```

It is off by default because those headers also block third-party images,
fonts, and iframes that do not opt in with CORP. Turn it on for a `--wasm`
build; leave it off otherwise.

`.wasm` already gets `application/wasm` from `shelf_static`, so streaming
compilation works with no configuration.

## What this is not

This is not server-side rendering. The first response is the shell; the
application appears once its script has loaded and run. For Flutter web in
particular there is no pre-rendered first paint at all — it draws to canvas or
builds the DOM at runtime, and nothing renders that on the server.

If you need HTML in the first response — for a crawler, a preview card, or a
page that has to be readable before script runs — render it server-side with
[templates](rendering.md) and serve the Flutter application on the routes that
do not need it. They compose on the same router:

```dart
final app = Router()
  ..route('/', get(landingPage))        // rendered HTML, indexed
  ..nest('/api', apiRoutes)
  ..fallback(staticFiles('build/web', html: true)); // the application
```

## Checklist before a deploy

1. Point at build output — `build/web` for Flutter, `dist` for most bundlers
   — not at the source directory.
2. Put `staticFiles(..., html: true)` in `fallback`, after the API routes.
3. Terminate TLS upstream; see [Serving](serving.md).
4. If the application is behind a path prefix, tell the build about it
   (`--base-href=/prefix/` for Flutter, `base` for Vite) — the server does not
   rewrite the document.
5. Confirm `curl -I` shows `cache-control: no-cache` on `/` and on
   `/main.dart.js`, and `immutable` on something under `/assets/`.
