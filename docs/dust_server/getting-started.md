# Getting started

## Install

The package is unpublished, so depend on it by path from inside this workspace:

```yaml
dependencies:
  dust_server:
    path: ../../packages/dust_server
```

## Entry points

Import everything, or only the part a file needs. Every symbol is reachable
from `server.dart`; the narrower libraries exist so a composition file does not
pull in annotations, and a handler file does not pull in the server.

| Library | Contents |
| :--- | :--- |
| `package:dust_server/server.dart` | everything below |
| `package:dust_server/annotations.dart` | `@Controller`, the verbs, parameter annotations, `@Routes` |
| `package:dust_server/extraction.dart` | `FromRequestParts`, `FromRequest`, the built-in extractors, coercion |
| `package:dust_server/response.dart` | `Rejection`, `IntoResponse`, encoders, `guard`, `ServerErrors` |
| `package:dust_server/router.dart` | `Router`, `Route`, `MethodRouter`, the verb builders, `Layer` |
| `package:dust_server/layers.dart` | `RequestTimeout`, `RequestId`, `AccessLog` |
| `package:dust_server/serving.dart` | `serveRouter`, `serveCluster`, static files |
| `package:dust_server/ws.dart` | WebSocket routes and sessions |
| `package:dust_server/templating.dart` | template engines and HTML responses |

`test/libraries_test.dart` holds each one to that contract: a file importing
only `router.dart` has to compile without `server.dart`.

## Layout

`lib/src` is split by feature, and no file passes 180 lines. `test` mirrors it.

| Folder | Contents |
| :--- | :--- |
| `annotations/` | what the generator will read; nothing here has behavior |
| `request/` | `RequestParts`, and the coercion every keyed extractor uses |
| `extraction/` | the extractor interfaces and one file per built-in |
| `response/` | rejections, encoders, HTML, error reporting |
| `router/` | the router, matcher, composer, flattener, path helpers |
| `layers/` | the middleware that ships |
| `serving/` | running, draining, clustering, static files |
| `template/` | the template seam and its adapter |
| `ws/` | WebSocket routes and sessions |

Each folder carries a barrel, so a public entry point is one export line and the
folder decides what stays internal.

## Trying it

```bash
cd packages/dust_server
dart run example/hello_world.dart
```

```bash
curl -s localhost:8080/           # Hello, world!
curl -s localhost:8080/hello/ada  # Hello, ada!
curl -s localhost:8080/json       # {"greeting":"Hello, world!"}
```

A handler returns the value it produced and the verb builder encodes it: a
`String` goes out as `text/plain`, and anything else as JSON. That is axum's
rule. To send a JSON string specifically, wrap it — `['Hello']` or
`{'greeting': 'Hello'}` — or call `jsonResponse`.

Every example runs the same way. The
[index](../../packages/dust_server/example/README.md) lists all 51 by the
question each one answers.
