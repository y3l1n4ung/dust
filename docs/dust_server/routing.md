# Routing

The router is this package's own. `shelf_router` is **not a dependency**: it
was doing only the final lookup, while this package already computed 405s, path
patterns, and the `HEAD` fallback, and getting path parameters back out of it
meant reading its private `shelf_router/params` context key.

It stays a **dev dependency**, so `test/router/conformance/` can check this
router against it on 1024 paths and `test/router/integration/` can prove the two
still compose. Nothing in `lib/` imports it.

## Building

```dart
final app = Router()
  ..route('/health', get(health))
  ..route('/todos', get(listTodos).post(createTodo))
  ..route('/todos/{id}', get(readTodo).delete(deleteTodo))
  ..nest('/api/v1', apiRoutes)
  ..mount('/assets', staticFiles('web/assets'))
  ..layer(const RequestId())
  ..withState(repository)
  ..fallback(singlePageApp('web/index.html'));
```

| Method | Purpose |
| :--- | :--- |
| `route(path, methods)` | serve a path, one handler per method |
| `nest(path, router)` | mount another router under a prefix |
| `merge(router)` | mount another router at this level |
| `mount(path, handler)` | hand a whole subtree to any `shelf` handler |
| `layer(middleware)` | wrap everything below |
| `withState(value)` | attach a dependency, read by type |
| `fallback(handler)` | answer what nothing matched |
| `describe()` | list every route, for tooling |

## Describing the route table

`describe()` returns every route the application serves, with its method, its
path with `{name}` placeholders intact, and whatever metadata was attached to the
routers it was mounted through.

```dart
for (final route in app.describe()) {
  print('${route.method} ${route.path}');
}
```

Metadata is `Object`, not a named type, and that is the point. An OpenAPI
generator, a permissions audit, and a `routes` command each want something
different there, so each attaches its own type and takes it back with
`metadataOf<T>()`:

```dart
final app = Router(metadata: const ApiDoc(tag: 'todos'))
  ..route('/todos', get(listTodos));

app.describe().first.metadataOf<ApiDoc>();   // the ApiDoc
app.describe().first.metadataOf<Audit>();    // null — someone else's type
```

Naming one of those types in `dust_server` would make the runtime depend on a
format it has no business knowing, so nothing is named. The slot stays open for
OpenAPI support to land later without a breaking change.

What `describe()` is not is a schema source. Parameter types and request or
response bodies come from the build-time IR, not from the route table.

## Path syntax

| Form | Matches |
| :--- | :--- |
| `/todos` | exactly that path |
| `/todos/{id}` | one segment, captured as `id` |
| `/todos/{id\|[0-9]+}` | one segment matching the pattern, captured |
| `/files/{*rest}` | the whole remainder, captured |

Braces follow axum. A constrained parameter's pattern is inserted as written, so
one that allows `/` spans segments; `[^/]+` keeps it to one.

A declared trailing slash is kept: `/a` and `/a/` are different paths.

## Which route runs

The first route in declaration order whose path matches and whose method applies
wins. One rule, not a specificity ladder, which is what keeps mounts and
any-method routes predictable. It is also what `shelf_router` does.

Two routes that could never both run are refused when the handler is built:

```
duplicate route: GET /todos
unreachable route: GET /a/{name} is shadowed by GET /a/{id}
```

## Methods

`get`, `post`, `put`, `patch`, `delete`, `head`, `options`, and `any` build a
`MethodRouter`, which chains:

```dart
app.route('/todos', get(list).post(create).any(rejectEverythingElse));
```

A `HEAD` with no handler falls back to the `GET` route. A known path reached
with an unhandled method answers 405 with `Allow`, where `shelf_router` answers
404.

## Speed

Routes bucket by their whole literal prefix, static paths match by string
compare rather than pattern, and a request whose segment count cannot fit is
rejected before any pattern runs.

| Table | Per match |
| :--- | :--- |
| 100 dynamic routes | 29us |
| 1000 dynamic routes | 13us |
| 500 under one prefix | 11us |
| 500 static routes | 6us |

What still scans linearly is one bucket: routes identical up to their first
parameter compete. A trie would fix that; the numbers do not yet justify it.

## Sealing

Reading `handler` flattens the tree, caches the result, and closes the whole
subtree. Changing a router afterwards throws rather than producing a handler
that ignores the change.

## Trying it

```bash
dart run example/routing.dart
```

```bash
# nest: the inner routes do not repeat the prefix
curl -s localhost:8080/api/notes        # ["first","second"]
curl -s localhost:8080/api/notes/7      # {"id":"7"}

# merge: folded in at the same level, no prefix invented for it
curl -s localhost:8080/health           # {"status":"ok"}

# two verbs chained onto one path
curl -s -X POST localhost:8080/api/notes  # 201 {"created":true}

# 405, and the methods the path does serve
curl -i -X PUT localhost:8080/api/notes

# the fallback, rather than a bare 404
curl -s localhost:8080/nothing-here     # {"error":"no such route"}

# HEAD falls back to the GET handler
curl -I localhost:8080/health
```

The 405 is the part worth reading:

```
HTTP/1.1 405 Method Not Allowed
allow: GET, HEAD, POST

{"error":"PUT is not allowed on /api/notes"}
```

A path that exists but not for this method is never a 404 — the difference tells
a client whether to fix the verb or the URL.
