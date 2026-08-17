# dust_server

Runtime support for Dust-generated Dart HTTP servers on `shelf`.

> [!IMPORTANT]
> Unreleased and unpublished (`publish_to: none`). The code generator described
> in [docs/design/server-plugin.md](../../docs/design/server-plugin.md) does not
> exist yet; this package is the runtime that generated code will call, built
> first so the design is checked against working code rather than a sketch.

## Documentation

Full documentation lives in [docs/dust_server](../../docs/dust_server/README.md).

| Section | Covers |
| :--- | :--- |
| [Getting started](../../docs/dust_server/getting-started.md) | the smallest server, entry points, layout |
| [Routing](../../docs/dust_server/routing.md) | paths, methods, nesting, mounting, matching rules, speed |
| [Extraction](../../docs/dust_server/extraction.md) | the built-in extractors, coercion, writing one |
| [Responses](../../docs/dust_server/responses.md) | encoders, rejections, `guard`, error reporting |
| [WebSockets](../../docs/dust_server/websockets.md) | upgrades, sessions, close codes |
| [Rendering](../../docs/dust_server/rendering.md) | templates, static files, single-page apps |
| [Serving](../../docs/dust_server/serving.md) | draining, TLS, isolate clustering |
| [Middleware](../../docs/dust_server/middleware.md) | layers, ordering, what ships |
| [Testing](../../docs/dust_server/testing.md) | running the suite, testing an application |
| [Dependencies](../../docs/dust_server/dependencies.md) | what is reused, and what is not |

## At a glance

```dart
final app = Router()
  ..layer(const RequestId())
  ..route('/todos/{id}', get(readTodo))
  ..route('/chat/{room}', ws(joinRoom))
  ..mount('/assets', staticFiles('web/assets'))
  ..withState(repository)
  ..fallback(singlePageApp('web/index.html'));

final server = await serveRouter(app, InternetAddress.anyIPv4, 8080);
```

HTTP, WebSockets, server-rendered HTML, static assets, and isolate clustering,
all on one router.

## Examples

| Example | Shows |
| :--- | :--- |
| [`example/todo_api.dart`](example/todo_api.dart) | a JSON API: extraction, validation, typed failures, layers, draining |
| [`example/chat_server.dart`](example/chat_server.dart) | HTTP, WebSockets, and rendered HTML from one route table |
| [`example/auth_schemes.dart`](example/auth_schemes.dart) | bearer, API key, HTTP Basic, and session cookie behind one interface |

Both are served over a loopback socket by `test/example/` and driven with a
real client.

One more lives in the workspace, where `dust build` can reach it:

| Example | Shows |
| :--- | :--- |
| [`examples/todo_server`](../../examples/todo_server) | generated models, a SQLite store, ownership rules, stress tests |

## Tests

834 tests, 100% line coverage.

```bash
dart test
```
