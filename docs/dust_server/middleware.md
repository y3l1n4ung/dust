# Middleware

A layer wraps everything below the router it is added to.

```dart
final app = Router()
  ..layer(const RequestTimeout(Duration(seconds: 30)))
  ..layer(const RequestId())
  ..layer(AccessLog(logger.info))
  ..nest('/api', apiRoutes);
```

## Order

Layers run outermost first, in the order they were added, down the tree: a layer
on the root runs before one on a nested router, which runs before one on the
module it holds. A layer on a nested router never runs for a sibling branch.

The root's layers wrap the router itself, so they also cover requests that
matched nothing.

## What ships

Nothing is installed by default.

| Layer | Purpose |
| :--- | :--- |
| `RequestTimeout(budget, {onTimeout})` | answers 503 past the budget |
| `RequestId({header, generate})` | keeps an inbound `x-request-id` or assigns a UUID, and echoes it |
| `AccessLog(onRecord)` | reports method, path, status, duration, and request id |

`AccessLog` prints nothing itself. What a log line looks like, and where it goes,
is the application's decision.

`RequestTimeout` bounds how long a handler may take, which the body limit does
not. It cannot **cancel** the handler: Dart has no cancellation, so the work
keeps running with nobody waiting for it.

## Trying it

One example per layer, each under 60 lines. The two worth running first are the
ones where placement changes the answer.

**`routeLayer` versus `layer`** — `dart run example/route_layer.dart`:

```bash
curl -si localhost:8080/admin/orders
curl -s  localhost:8080/admin/orders -H 'authorization: Bearer staff'
curl -si localhost:8080/admin/typo
curl -s  localhost:8080/health
```

```
401 WWW-Authenticate: Bearer   {"error":"expected a bearer token"}
200                            ["order-1"]
404                            {"error":"no route for /admin/typo"}
200                            {"status":"ok"}
```

The third line is the point. A guard added with `layer` answers **401** there, so
a typo in your own route table looks like an auth failure — and you spend the
afternoon on the credential instead of the spelling.

**A nested `layer` is scoped to its prefix** —
`dart run example/normalize_path.dart`:

```bash
curl -s localhost:8080/notes/      # rewritten, one response, same URL
curl -s localhost:8080/api/notes/  # nested, covered from above
```

A `layer` covers everything its router answers — the 404s and 405s inside its
prefix included — so it can run *before* routing, which is what a path rewrite
depends on. `buildScopedApp` in that example puts the layer on `/api` alone:

```
GET /api/notes/  -> 200 ["first"]
GET /shop/notes/ -> 404 {"error":"no route for /shop/notes/"}
```

Scoping earns its keep when one half of an application has URLs you must not
rewrite — a webhook whose signature is computed over the exact path, say.

**The rest**, each with its own example:

| Layer | Example | The bit worth reading |
| :--- | :--- | :--- |
| `Cors` | `cors.dart` | a preflight answers 204 without reaching a handler; `Vary: Origin` is set; credentials with `any()` throws at construction |
| `Compression` | `compression.dart` | `gzip;q=0` is a refusal, not an absence; under 1 KB is left alone |
| `RequestId` | `request_id.dart` | a client-supplied id is kept, so one id spans every hop |
| `AccessLog` | `access_log.dart` | records the 404 too, and the recorded path carries no query string |
| `SecurityHeaders` | `security_headers.dart` | CSP and HSTS have no defaults, and why; two nested routers scope one policy to each half |
| `RequestTimeout` | `request_timeout.dart` | the 503 goes back while the handler keeps running |

```bash
curl -sI localhost:8080/rows -H 'accept-encoding: gzip'      # content-encoding: gzip
curl -sI localhost:8080/rows -H 'accept-encoding: gzip;q=0'  # no encoding
curl -si localhost:8080/slow                                 # 503 after ~200ms
```

`AccessLog` is the one to point somewhere real. It hands you an `AccessRecord`
rather than printing, so a service that wants JSON on stdout, a counter, and a
sampled trace does not have to fork the layer to get them.

## Writing one

Two forms are accepted. A bare `shelf` `Middleware`:

```dart
app.layer(logRequests());
```

Or a `Layer`, which is const-expressible so a generated annotation can carry it:

```dart
final class RateLimit implements Layer {
  const RateLimit({required this.perMinute});

  final int perMinute;

  @override
  Middleware toMiddleware() => ...;
}
```

Anything else is refused when the handler is built, naming the offending type.
