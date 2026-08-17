# todo_server

A to-do API built from Dust-generated models on the `dust_server` runtime.

```bash
dart pub get
cargo run -p dust_cli -- build --root examples/todo_server
dart test
dart run bin/server.dart
```

## What the generator writes today

The models under `lib/models/` carry `@Derive` and `@Validate` and nothing
else. `dust build` writes three things into their part files, and each one is
what a layer of the runtime consumes:

| Generated | Consumed by |
| :--- | :--- |
| `serialize` | `jsonResponse`, which writes any `Serializable` |
| `deserialize` | `JsonExtractable`, given the function by name |
| `validate` | `ValidatedExtractable`, which turns failures into a 422 |
| `fromRow` | the generated `@Query` methods, which return `Todo` directly |

`Todo` carries all four. It is the row the database returns, the JSON the API
answers with, and the value a request decodes into — one class, because here
they are genuinely the same shape. A separate row type earns its keep once the
table and the resource start to differ.

So a create endpoint never mentions JSON:

```dart
const createTodoBody = ValidatedExtractable<CreateTodo>(
  JsonExtractable<CreateTodo>(CreateTodo.deserialize),
);
```

A body that breaks a constraint answers with the field and the message from
the annotation, not one flat string:

```json
{"error": "Validation failed", "fields": {"title": ["must be 1 to 200 characters"]}}
```

## What the handlers still spell out

`lib/src/handlers.dart` and `lib/src/app.dart` are hand-written. Every endpoint
costs a handler that runs its extractors in order and a route binding that
names it:

```dart
Future<Result<Todo, Rejection>> readTodo(Request request) async {
  await const BearerAuth().require(request);
  final id = await path<String>('id').require(request);
  final repository = await state<TodoRepository>().require(request);

  final todo = repository.find(id);
  return todo == null ? const Err(_missing) : Ok(todo);
}

..route('/{id}', get(readTodo).patch(completeTodo).delete(deleteTodo))
```

The extractors are still listed by hand and the route is still bound by hand.
Read that as a reference for the shape a generator has to produce, not as a
claim about how much a backend author should have to type.

## Deliberately low-level

Everything here uses the runtime directly, so the example stays runnable with
no generator step beyond `derive`. Nothing in this package depends on a server
plugin, because there isn't one.
