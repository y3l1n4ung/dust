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

```bash
dart run example/cors.dart
```

```bash
# every response carries an id, whatever it answered
curl -sD- -o/dev/null localhost:8080/health | grep -i x-request-id
curl -sD- -o/dev/null localhost:8080/api/v1/todos | grep -i x-request-id

# the layers wrap the failures too, so a 401 is still traced and logged
curl -sD- -o/dev/null localhost:8080/api/v1/todos | grep -iE 'x-request-id|traceparent'
```

The access log line and the span appear on the server's stdout for each.

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
