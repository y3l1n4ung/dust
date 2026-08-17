# Testing

## Running

```bash
dart test packages/dust_server
```

```bash
dart analyze --fatal-infos packages/dust_server
```

Coverage:

```bash
bash scripts/dart/coverage.sh packages/dust_server 100
```

Currently **100%** of lines, 1204 of 1204, and the same script runs in CI, so a
change that drops coverage fails the build and names the uncovered lines.

The raw commands, if you want the lcov report itself:

```bash
cd packages/dust_server
dart test --coverage=.coverage
dart run coverage:format_coverage --lcov --in=.coverage --out=.coverage/lcov.info --report-on=lib
```

## How this package is tested

834 tests, organized so a gap looks like a thin folder rather than hiding inside
a long file.

| Folder | Covers |
| :--- | :--- |
| `test/router/matching/` | placeholders, encoding, literals, trailing slashes, declaration order |
| `test/router/methods/` | HEAD fallback, 405 and `Allow`, every verb builder, `any` |
| `test/router/composition/` | nest, merge, duplicates, shadowing, sealing, fallback, describe |
| `test/router/patterns/` | catch-alls and constrained parameters |
| `test/router/mounting/` | handing a subtree to another handler |
| `test/router/middleware/` | layer order and layer types |
| `test/router/state/` | scoping, override, missing state |
| `test/router/lifecycle/` | body limits, error reporting |
| `test/router/conformance/` | parity with `shelf_router`, hand-picked and randomized |
| `test/router/integration/` | real socket, `shelf` interop, concurrency, throughput |
| `test/router/hardening/` | pathological input, untrusted request fields |
| `test/extraction/`, `test/request/`, `test/response/` | one file per extractor and encoder |
| `test/layers/`, `test/serving/`, `test/ws/`, `test/templating/` | the rest of the surface |
| `test/generated/` | what the plugin will emit, hand-written until it exists |
| `test/example/` | the example application, over a socket |

Three habits are worth copying:

- **Differential testing.** The router is checked against `shelf_router` on 24
  hand-picked paths and 1000 generated ones. Every intentional difference is
  asserted rather than left to be noticed later.
- **Real sockets.** Anything about HTTP is tested through a real client on a
  real port. In-memory `Request` objects hide encoding bugs.
- **Pinning surprises.** When a dependency behaves unexpectedly, the test
  records the real behavior with a comment saying why, instead of asserting what
  was assumed.

## Testing an application

A `Router`'s `handler` is a plain `shelf` handler, so most tests need no socket:

```dart
final app = Router()..route('/todos', get(listTodos));

final response = await app.handler(
  Request('GET', Uri.parse('http://localhost/todos')),
);

expect(response.statusCode, 200);
```

For anything touching encoding, upgrades, or shutdown, serve it:

```dart
final server = await serveRouter(app, InternetAddress.loopbackIPv4, 0);
addTearDown(server.close);

final response = await http.get(
  Uri.parse('http://${server.address.host}:${server.port}/todos'),
);
```

Port `0` binds a free port, so tests can run in parallel.

## Driving a server by hand

The suite uses a real socket, and so can you. Every example takes the same
shape: `buildApp` is separate from `main`, so a test serves it on port 0 and a
terminal serves it on a fixed one.

```bash
dart run example/todo_api.dart
```

```bash
# what the flow tests assert, one request at a time
curl -s -H 'authorization: Bearer todos:read' localhost:8080/api/v1/todos
curl -s -X POST localhost:8080/api/v1/todos \
  -H 'authorization: Bearer todos:write' \
  -H 'content-type: application/json' --data '{"title":"buy milk"}'
curl -i -X DELETE -H 'authorization: Bearer todos:write' \
  localhost:8080/api/v1/todos/2
```

Add `-i` to see the status and headers, `-v` to see the request as it goes out.
When a test fails, running the same request by hand is usually faster than
reading the diff.
