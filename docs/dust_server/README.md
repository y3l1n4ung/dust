# dust_server

Runtime for Dart HTTP servers: routing, extraction, responses, WebSockets,
server-rendered HTML, static files, and serving across isolates.

> [!IMPORTANT]
> Unpublished (`publish_to: none`) and pre-1.0. The generator described in
> [Server Plugin Design](../design/server-plugin.md) does not exist yet; this is
> the runtime generated code will call, built first so the design is checked
> against working code.

## Sections

| Document | What it covers |
| :--- | :--- |
| [Tutorial](tutorial.md) | a to-do API built one endpoint at a time — start here |
| [Getting started](getting-started.md) | the smallest working server, and where each piece lives |
| [Routing](routing.md) | `Router`, paths, methods, nesting, mounting, matching rules |
| [Extraction](extraction.md) | turning a request into handler arguments, and the failures that produces |
| [Authentication](authentication.md) | credential extractors, schemes, challenges, composing |
| [Responses](responses.md) | rejections, status codes, encoders, error reporting |
| [WebSockets](websockets.md) | upgrades on the same router, sessions, close codes |
| [Rendering](rendering.md) | templates, server-rendered HTML, static files, single-page apps |
| [Web applications](web-apps.md) | HTML mode: client-side routes, cache policy, cross-origin isolation |
| [Serving](serving.md) | running, draining, TLS, and isolate clustering |
| [Middleware](middleware.md) | layers, `routeLayer`, CORS, compression, and the rest |
| [Tracing](tracing.md) | spans, W3C trace context, exporters |
| [Testing](testing.md) | how the package is tested, and how to test an application built on it |
| [Dependencies](dependencies.md) | what is reused, and why each was chosen over writing it |
| [Production readiness](../design/server-production-readiness.md) | what is proven, what is missing |

## The shortest version

```dart
import 'dart:io';

import 'package:dust_server/server.dart';

Future<void> main() async {
  final app = Router()
    ..layer(const RequestId())
    ..route('/todos/{id}', get(readTodo))
    ..withState(TodoRepository());

  final server = await serveRouter(app, InternetAddress.anyIPv4, 8080);
  await ProcessSignal.sigterm.watch().first;
  await server.close(drain: const Duration(seconds: 15));
}

Future<Result<Todo, Rejection>> readTodo(Request request) async {
  final id = await request.path<String>('id');
  final repository = await request.state<TodoRepository>();

  final todo = repository.find(id);
  return todo == null ? const Err(Rejection.notFound('no such todo')) : Ok(todo);
}
```

An endpoint reads what it needs and returns what it produced. Each read throws
the rejection its extractor produced, so the first failure ends the endpoint.
Converting the result, catching what escapes, and choosing the status happen
once, in the verb builder — which is generic over the return type, so
answering with the wrong model is a compile error.

Complete applications, smallest first:

| Example | Shows |
| :--- | :--- |
| [`packages/dust_server/example/todo_api.dart`](../../packages/dust_server/example/todo_api.dart) | a JSON API on the runtime alone, with no generator step |
| [`packages/dust_server/example/chat_server.dart`](../../packages/dust_server/example/chat_server.dart) | HTTP, WebSockets, and server-rendered HTML from one route table |
| [`examples/todo_server`](../../examples/todo_server) | generated models, a SQLite store, and ownership rules |

Every one of them is driven over a real socket by its own tests.
