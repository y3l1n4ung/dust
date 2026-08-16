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
