# Responses

## Encoders

| Function | Produces |
| :--- | :--- |
| `jsonResponse(value, {status})` | `application/json` |
| `textResponse(value, {status})` | `text/plain; charset=utf-8` |
| `htmlResponse(markup, {status})` | `text/html; charset=utf-8` |
| `bytesResponse(bytes, {status})` | `application/octet-stream` |
| `streamResponse(stream, {status})` | a streamed octet-stream |
| `noContent()` | 204 |

## Rejections

`Rejection` is a failure with a status code, and it turns itself into a
response. Body failures keep their own codes rather than collapsing into one
400, because a client can act on the difference.

| Failure | Status |
| :--- | :--- |
| Missing or wrong `content-type` | 415 |
| Malformed JSON syntax | 400 |
| Well-formed JSON, wrong shape | 422 |
| Path or query coercion failure | 400 |
| Body past the limit | 413 |
| Validation failure | 422, with field errors |
| Missing state, or a context value nobody wrote | 500 |

A 401 carries `WWW-Authenticate`, defaulting to `Bearer`; the specification
requires it, and a bare JSON body is not a valid 401. A known path reached with
an unhandled method answers 405 with `Allow`.

## Turning a value into a response

```dart
final class NotFound implements IntoResponse {
  const NotFound(this.message);

  final String message;

  @override
  Response intoResponse() => jsonResponse({'error': message}, status: 404);
}
```

## Failures inside a handler

```dart
return guard(() async {
  final todo = await repository.create(input);
  return jsonResponse(todo.toJson(), status: 201);
});
```

`guard` turns a thrown `Rejection` into its own response, so a guard clause can
throw instead of threading a `Result` back, and anything else into a 500 whose
body says nothing. The error and stack go to the router's `onError`.

## Where errors are reported

```dart
final app = Router(onError: (error, stack) => log.severe('handler failed', error));
```

The sink is scoped to a zone, not the process, so two routers in one isolate
keep their own. `ServerErrors.reporter` sets a process-wide fallback for
anything a router did not claim.

## Trying it

```bash
dart run example/todo_api.dart
```

Every failure the API can answer with, in the order a client meets them:

```bash
# 401, with the challenge the specification requires
curl -i localhost:8080/api/v1/todos

# 403: authenticated, but the token lacks todos:write
curl -i -X POST localhost:8080/api/v1/todos \
  -H 'authorization: Bearer todos:read' \
  -H 'content-type: application/json' --data '{"title":"x"}'

# 404
curl -i -H 'authorization: Bearer todos:read' localhost:8080/api/v1/todos/nope

# 415: the body extractor wanted JSON
curl -i -X POST localhost:8080/api/v1/todos \
  -H 'authorization: Bearer todos:write' \
  -H 'content-type: text/plain' --data 'x'

# 422 naming the field and repeating the message from the annotation
curl -i -X POST localhost:8080/api/v1/todos \
  -H 'authorization: Bearer todos:write' \
  -H 'content-type: application/json' --data '{"title":""}'

# 204, with no body at all
curl -i -X DELETE -H 'authorization: Bearer todos:write' \
  localhost:8080/api/v1/todos/1
```

A 500 never carries the detail: the message goes to the router's `onError` and
the client gets `{"error":"Internal server error"}`.
