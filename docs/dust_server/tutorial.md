# Tutorial

Build a to-do API one endpoint at a time. Every snippet runs. For the same
pieces isolated one per file, see the
[examples index](../../packages/dust_server/example/README.md).

```yaml
# pubspec.yaml
dependencies:
  dust_dart: ^0.1.3
  dust_server:
    path: ../../packages/dust_server
```

## 1. The smallest server

An endpoint returns what it produced. The verb builder turns that into a
response:

```dart
import 'dart:io';

import 'package:dust_server/server.dart';

Future<void> main() async {
  final app = Router()..route('/health', get(health));

  await serveRouter(app, InternetAddress.loopbackIPv4, 8080);
}

Future<Map<String, Object?>> health(Request request) async => {'ok': true};
```

```bash
curl localhost:8080/health
# {"ok":true}
```

The map became JSON because the endpoint returned it. There is no encoder
call, no `Response`, and nothing catching errors. `get` is generic over the
return type, so an endpoint that answers with the wrong thing is a compile
error.

## 2. Path parameters

`{id}` in the route captures a segment; `request.path<String>('id')` reads it.
Each call throws the rejection its extractor produced, so the first failure
ends the endpoint and nothing after it runs:

```dart
..route('/todos/{id}', get(readTodo))

Future<Map<String, Object?>> readTodo(Request request) async {
  final id = await request.path<String>('id');

  return {'id': id};
}
```

The type argument is a coercion, not a cast. `path<int>('id')` on `/todos/abc`
answers 400 before the handler runs, and the handler receives a real `int`.

## 3. Query parameters

A non-nullable type makes the value required; a nullable one makes it optional:

```dart
Future<Map<String, Object?>> listTodos(Request request) async {
  final done = await request.query<bool?>('done');  // null when absent
  final limit = await request.query<int>('limit');  // 400 when absent

  return {'done': done, 'limit': limit};
}
```

## 4. Dependencies

What a controller class would hold in a field is attached to the router and
read back by type — the same pairing axum's `State` has with `with_state`:

```dart
final class TodoRepository {
  final _todos = <String, Todo>{};

  Todo? find(String id) => _todos[id];
}

final app = Router()
  ..route('/todos/{id}', get(readTodo))
  ..withState(TodoRepository());

Future<Todo?> readTodo(Request request) async {
  final id = await request.path<String>('id');
  final repository = await request.state<TodoRepository>();

  return repository.find(id);
}
```

Nothing names a key on either side. Forgetting `withState` is a 500 that says
which type is missing, not a null dereference.

## 5. Models

Annotate a model and let `dust build` write its JSON:

```dart
import 'package:dust_dart/serde.dart';

part 'todo.dart.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
final class Todo with _$Todo {
  const Todo({required this.id, required this.title, required this.done});

  static Todo deserialize(Map<String, Object?> json) => _$TodoDeserialize(json);

  final String id;
  final String title;
  final bool done;
}
```

```bash
cargo run -p dust_cli -- build --root .
```

An endpoint can now declare `Future<Todo>` or `Future<List<Todo>>` and return
the model itself. Anything deriving `Serialize` is written through its
generated `serialize`, including nested values and list elements.

## 6. Request bodies

`request.body` takes the generated `deserialize` by name, and `validBody`
decodes and then runs the generated `validate`:

```dart
..route('/todos', post(createTodo, status: 201))

Future<Todo> createTodo(Request request) async {
  final repository = await request.state<TodoRepository>();
  final input = await request.validBody(CreateTodo.deserialize);

  return repository.add(input);
}
```

`status` is the success status. A failure keeps its own, so a create that
rejects still answers 422 rather than 201.

## 7. Validation

Add constraints to the model and wrap the extractor in `valid`:

```dart
@Derive([Deserialize(), Validate()])
final class CreateTodo with _$CreateTodo {
  const CreateTodo({required this.title, required this.owner});

  static CreateTodo deserialize(Map<String, Object?> json) =>
      _$CreateTodoDeserialize(json);

  @Validate(length: Length(min: 1, max: 200), message: 'must be 1 to 200 characters')
  final String title;

  @Validate(email: true, message: 'must be an email address')
  final String owner;
}
```

```dart
final input = await request.validBody(CreateTodo.deserialize);
```

A body that breaks a constraint answers 422 with the field and the message
from the annotation:

```json
{
  "error": "Validation failed",
  "fields": {
    "title": ["must be 1 to 200 characters"],
    "owner": ["must be an email address"]
  }
}
```

Every failure is reported, not just the first, and two failures on one field
stay two entries. A body of the *wrong shape* is a different failure: also 422,
but with no `fields`, because no single field is to blame.

> A Dart constructor default does not reach deserialization. Add
> `@SerDe(defaultValue: false)` for a field a request may omit.

## 8. Custom extractors

Anything implementing `FromRequestParts<T>` is an extractor, run with
`request.extract(...)`. Configuration goes in the constructor:

```dart
class BearerAuth implements FromRequestParts<AuthUser> {
  const BearerAuth({this.scope});

  final String? scope;

  @override
  Future<Result<AuthUser, Rejection>> extract(Request request) async {
    final raw = RequestParts.of(request).headers['authorization'];
    if (raw == null || !raw.startsWith('Bearer ')) {
      return const Err(Rejection.unauthorized('missing bearer token'));
    }

    final scopes = raw.substring(7).split(',');
    if (scope != null && !scopes.contains(scope)) {
      return Err(Rejection.forbidden('requires scope $scope'));
    }
    return Ok(AuthUser(scopes.first, scopes));
  }
}

final class TodosWrite extends BearerAuth {
  const TodosWrite() : super(scope: 'todos:write');
}
```

Put it first and nothing after it runs when it rejects:

```dart
Future<Todo> createTodo(Request request) async {
  await request.extract(const TodosWrite());
  final repository = await request.state<TodoRepository>();
  final input = await createTodoBody;

  return repository.add(input);
}
```

An unauthorized request never reaches the body, so a 401 costs nothing.

## 9. Answering failures

Return the failure; do not throw it and do not build a response. Putting it in
the return type keeps it in the signature rather than in a comment:

```dart
Future<Result<Todo, Rejection>> readTodo(Request request) async {
  final id = await request.path<String>('id');
  final repository = await request.state<TodoRepository>();

  final todo = repository.find(id);
  return todo == null ? const Err(Rejection.notFound('no such todo')) : Ok(todo);
}
```

`Result<Null, Rejection>` is the delete case: `Ok(null)` answers 204, so an
empty answer is deliberate rather than accidental.

The verb builders are generic over the return type, so the compiler holds you
to whatever you declared:

| Return | Answer |
| :--- | :--- |
| a model, list, or map | 200 (or `status`) as JSON |
| `null` | 204 with no body |
| a `Rejection` | its own status, as JSON |
| `Ok(value)` | the value |
| `Err(error)` | the error, defaulting to 500 |
| `Some(value)` / `None` | the value / 404 |
| a `Response` | itself, untouched |

Anything **thrown** that is not a `Rejection` becomes an opaque 500, with the
real error going to the router's `onError` instead of to the client.

## 10. Composing the application

Prefixes nest, and layers apply to everything below them:

```dart
final todos = Router()
  ..route('/', get(listTodos).post(createTodo, status: 201))
  ..route('/{id}', get(readTodo).patch(completeTodo).delete(deleteTodo));

final app = Router(onError: (error, stack) => stderr.writeln(error))
  ..layer(const RequestTimeout(Duration(seconds: 10)))
  ..layer(const RequestId())
  ..layer(AccessLog(stdout.writeln))
  ..nest('/api/v1', Router()..nest('/todos', todos))
  ..route('/health', get(health))
  ..withState(repository);
```

A method the path does not serve answers 405 with `Allow`; a path nothing
serves answers 404. `HEAD` falls back to `GET`.

## 11. Running it

```dart
final server = await serveRouter(app, InternetAddress.anyIPv4, 8080);

await ProcessSignal.sigint.watch().first;
await server.close(drain: const Duration(seconds: 10));
```

`close` stops accepting and waits for in-flight requests within the deadline.
For several isolates on one port, see [Serving](serving.md).

## Where to go next

| Next | For |
| :--- | :--- |
| [Extraction](extraction.md) | every built-in extractor and its rejections |
| [Responses](responses.md) | encoders, `IntoResponse`, error reporting |
| [Routing](routing.md) | matching rules, mounting, `shelf` interop |
| [WebSockets](websockets.md) | upgrades on the same router |
| [Rendering](rendering.md) | templates, static files, single-page apps |
| [Testing](testing.md) | driving an application over a real socket |
