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

The failures a body extractor can answer with, in the order a client meets them.
`dart run example/json_body.dart`:

```bash
# 415: the extractor wanted JSON and got a form
curl -i -X POST localhost:8080/notes -d 'title=x'

# 400: the content-type was right, the bytes were not JSON
curl -i -X POST localhost:8080/notes -H 'content-type: application/json' \
  -d 'not json'

# 422: valid JSON of the wrong shape
curl -i -X POST localhost:8080/notes -H 'content-type: application/json' \
  -d '{"body":"no title"}'
```

```
415 {"error":"expected application/json"}
400 {"error":"malformed JSON: Unexpected character"}
422 {"error":"JSON body does not match the expected shape: FormatException: title is required"}
```

Three different statuses for three different mistakes. A client that gets 400
knows to fix its encoder; 415, its header; 422, its field.

**What the 422 says is up to your deserializer.** The message on the throw
reaches the client, so `json['title']! as String` answers "Null check operator
used on a null value" — a Dart implementation detail that names no field. A
generated `Deserialize()` names it for you; hand-written, name it yourself.

`dart run example/validation_422.dart` shows the other half — a payload that
decoded cleanly and broke a rule:

```bash
curl -s -X POST localhost:8080/products -H 'content-type: application/json' \
  -d '{"title":"","priceCents":0}'
```

```json
{
  "error": "Validation failed",
  "fields": {
    "title": ["is required"],
    "priceCents": ["must be more than zero"]
  }
}
```

Both fields, not the first — a client told to fix one field, then refused for the
field beside it, will make the round trip twice. Both failures also land on 422
with the same body shape, so one error renderer covers them.

To publish a different shape entirely, `dart run example/customize_rejection.dart`
reshapes every failure as RFC 9457 `application/problem+json`:

```bash
curl -s localhost:8080/orders/abc
curl -s localhost:8080/nothing
```

```json
{"type":"about:blank","title":"path parameter \"id\" is not a valid integer","status":400,"instance":"/orders/abc"}
{"type":"about:blank","title":"no route for /nothing","status":404,"instance":"/nothing"}
```

The second one is the reason to do this in a **layer** rather than per handler:
the router's own 404 was raised before any handler ran, and it still comes out in
the published shape.

A 500 never carries the detail: the message goes to the router's `onError` and
the client gets `{"error":"Internal server error"}`.
