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
dart run example/todo_api.dart
```

```bash
# the route table, one path at a time
curl -s -H 'authorization: Bearer todos:read' localhost:8080/api/v1/todos
curl -s -H 'authorization: Bearer todos:read' localhost:8080/api/v1/todos/1
curl -s localhost:8080/health

# 405, and the methods the path does serve
curl -i -X PUT -H 'authorization: Bearer todos:read' localhost:8080/api/v1/todos

# 404 for a path nothing matched
curl -i localhost:8080/api/v1/nothing

# HEAD falls back to the GET handler
curl -I localhost:8080/health
```

`-i` prints the status line and headers; the `Allow` header on the 405 is the
part worth reading.
