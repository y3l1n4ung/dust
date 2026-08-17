# notes_mvc

The smallest Dust server worth writing. **Five routes, 230 lines**, including
comments and the SQL.

```bash
dart pub get
cargo run -p dust_cli -- build --root examples/notes_mvc
cargo run -p dust_cli -- build --root examples/notes_mvc --db
dart test
dart run bin/server.dart
```

## Three files

| File | Is |
| :--- | :--- |
| [`lib/models/note.dart`](lib/models/note.dart) | **Model** — the note, and the draft a request may send |
| [`lib/db/database.dart`](lib/db/database.dart) | the connection: open, migrate, close |
| [`lib/db/notes_queries.dart`](lib/db/notes_queries.dart) | the queries, generated from `@Query` |
| [`lib/handlers/notes.dart`](lib/handlers/notes.dart) | **Controller** — one function per route |
| [`lib/app.dart`](lib/app.dart) | the routes |

The database and the queries are separate on purpose. A **database** is opened
once by `main` and closed on the way down; **queries** are bound to an executor
— `Notes(database.connection)` normally, `Notes(tx)` inside a transaction. One
type could not be both, and a handler has no business closing a connection.

The **view** is the return type: a `Note` becomes JSON through its generated
`serialize`, `null` becomes 204, a `Rejection` becomes its own status. Nothing
builds a response.

The **controller** is a function per route — no class, because a class would
only exist to hold the database, and the database comes from state.

## What is deliberately not here

No repository. No store interface. No row type separate from the model. **No
controller class either** — a handler is a plain function, and the queries
arrive the way everything else does:

```dart
Future<List<Note>> index(Request request) async {
  final notes = await request.state<Notes>();

  return _ok(await notes.listAll());
}

Future<Result<Note, Rejection>> show(Request request) async {
  final id = await request.path<int>('id');
  final notes = await request.state<Notes>();

  final note = _ok(await notes.findById(id));
  return note == null ? const Err(_missing) : Ok(note);
}
```

`request.state<Notes>()` is axum's `State<S>` and FastAPI's dependency, in
Dart. Nothing is captured in a closure or held in a field, so the database goes
in at one place:

```dart
Router buildApp(Notes queries) => Router()
  ..route('/notes', get(index).post(create, status: 201))
  ..withState(queries);
```

Swapping the database — a file, a fake, a transaction — is a change there and
nowhere else.

One class carries all three jobs — it is the row `FromRow` maps, the JSON
`Serialize` writes, and the shape `Validate` checks:

```dart
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize(), FromRow()])
final class Note with _$Note {
  const Note({required this.id, required this.title, required this.body});

  final int id;
  final String title;
  final String body;
}
```

Split those apart when they actually start to differ — when the API stops
serving what the table stores. [`examples/todo_server`](../todo_server) is what
that looks like: a store interface with two implementations, a row type distinct
from the resource, and ownership rules. It is the same runtime, five layers
deeper, because it needs to be.

## Trying it

```bash
dart run bin/server.dart
```

```bash
curl -s localhost:8083/notes
curl -s -X POST localhost:8083/notes \
  -H 'content-type: application/json' \
  --data '{"title":"buy milk","body":"today"}'
curl -s localhost:8083/notes/1
curl -s -X PUT localhost:8083/notes/1 \
  -H 'content-type: application/json' \
  --data '{"title":"buy oat milk","body":"today"}'
curl -i -X DELETE localhost:8083/notes/1
```

The failures are worth a look too:

```bash
# 400: the id is not a number, so no query runs
curl -i localhost:8083/notes/abc

# 422 naming the field and repeating the message from the annotation
curl -i -X POST localhost:8083/notes \
  -H 'content-type: application/json' --data '{"title":""}'

# 405, with the methods the path does serve
curl -i -X PUT localhost:8083/notes -H 'content-type: application/json' --data '{}'
```

## Adding a route

Three edits, in one direction:

1. A `@Query` in `notes_database.dart`.
2. A function in `handlers/notes.dart`.
3. A line in `buildApp`.

Run `dust build --db` and the query is checked against the schema before the
server starts.
