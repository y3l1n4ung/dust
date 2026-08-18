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

1264 tests, organized so a gap looks like a thin folder rather than hiding inside
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

## Two ways in, and when each is right

`example/testing.dart` shows both.

**`router.handler`** takes a `Request` and returns a `Response`, in process, no
socket:

```dart
final app = buildApp(NoteStore(['only']));

final response = await app.handler(
  Request('GET', Uri.parse('http://localhost/notes')),
);

expect(await response.readAsString(), '["only"]');
```

Fast enough to run thousands of, and right for statuses, bodies, and headers.

**`serveRouter` on port 0** goes over a real socket with a real client. Slower,
and the only way to catch what the wire does — gzip, chunked bodies, `HEAD`
dropping a body, a WebSocket upgrade:

```dart
final client = HttpClient()..autoUncompress = false;
final request = await client.getUrl(app.uri('/notes'));
request.headers.set('accept-encoding', 'gzip');

expect((await request.close()).headers.value('content-encoding'), 'gzip');
```

`autoUncompress` is off because `dart:io` otherwise decodes the body and strips
the header, and the assertion passes while proving nothing.

> **Port 0, never a fixed port.** The OS assigns a free one. A suite pinned to
> 8080 fails when anything else holds it — and worse, it can **pass** against
> another process that happens to be listening, measuring something that is not
> your code at all. That has happened here: a benchmark once reported numbers for
> a process started days earlier.

The other rule is in every example in this repository: `buildApp` takes its
dependencies, so a test hands in an empty store and asserts on it afterwards
rather than reaching for a global.

## Driving a server by hand

```bash
dart run example/testing.dart
```

```bash
curl -s  localhost:8080/notes
curl -s -X POST localhost:8080/notes \
  -H 'content-type: application/json' --data '{"title":"buy milk"}'
curl -si localhost:8080/notes/9
```

Add `-i` to see the status and headers, `-v` to see the request as it goes out.
When a test fails, running the same request by hand is usually faster than
reading the diff.
