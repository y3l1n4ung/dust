# Server Runtime: Production Readiness

Status: `packages/dust_server` is **not** production ready. `0.1.0-beta.1` is
on pub.dev, so this is now a document about a package other people can install
rather than an internal note. It says what is covered, what is missing, and
what would have to be true before an application is put in front of real
traffic.

For the feature itself, see the [Server Plugin Design](server-plugin.md).

## Where it stands

100% line coverage, gated in CI, and no analyzer findings. That says the code
does what its tests say. It does not say the tests ask the right questions,
which is what the rest of this document is about — and the record backs the
distinction: every defect below was found by writing an example or probing a
combination, never by the coverage number moving.

More than twenty defects have been found and fixed since, nearly all by writing
an example or probing a combination rather than by reading the code — which is
the strongest argument in this document for how the rest of it should be read.

Six of those came from one pass over deployment behaviour rather than API
behaviour, and three of the six shared a shape worth naming: a failure **after
the handler returned**, where `guard` cannot reach and the request's zone no
longer exists. A response header that could not be written hung the connection
with nothing logged; a server-sent events generator that threw never reached
`onError`; and a router factory that failed inside a spawned isolate hung
startup forever. That seam deserves a deliberate audit rather than another
round of probing.

## Covered

Router coverage is organized one folder per case under `test/router`, so a gap
is visible as a thin folder rather than hidden inside a long file:

| Folder | Cases |
| :--- | :--- |
| `matching/` | placeholders, percent encoding, literal segments, trailing slashes |
| `methods/` | HEAD fallback, 405 and `Allow`, every verb builder |
| `composition/` | nest, merge, duplicates, sealing, fallback, describe |
| `middleware/` | layer order down the tree, layer types, nested-layer scoping |
| `state/` | scoping and override, missing state |
| `lifecycle/` | body limit, error reporting |
| `paths/` | prefix normalization and joining |
| `conformance/` | parity with `shelf_router`, hand-picked and randomized |
| `integration/` | real socket, `shelf` interop, concurrency, throughput |
| `mounting/` | mount, root mount, declaration order against a catch-all |
| `hardening/` | pathological input, untrusted request fields, decoded parameters |

Beyond the router, `test/example/` serves every example over a loopback socket
and drives each with a real client — 216 tests, one group per example, so an
example that stops compiling fails the build.

Three suites test properties no status code can show:

| Suite | Property |
| :--- | :--- |
| `response/streaming_test.dart` | events reach the client as they happen, read off a raw socket by arrival time |
| `serving/traversal_test.dart` | a static handler cannot be walked out of, sent byte for byte so no client normalizes the attempt away |
| `serving/background_tasks_test.dart` | work outliving a response is drained by shutdown, and detached from the request's span |

| Area | Evidence |
| :--- | :--- |
| Route matching | agrees with `shelf_router` on 24 hand-picked paths and 1000 generated ones |
| Real HTTP | serves over a socket and is driven by a real client: methods, status, path capture, percent-decoding, 404, 405, HEAD |
| Ecosystem fit | runs behind `shelf` middleware, hosts plain `shelf` handlers, mounts inside `shelf_router`, composes with `Cascade` |
| Concurrency | 200 simultaneous requests keep their own path parameters; the matcher is immutable after build |
| Pathological input | 10000-segment paths, 100 KB segments, and a 200-route table all answer in bounded time |
| Untrusted fields | a method carrying CRLF cannot forge a response header; paths are matched literally after Dart normalizes them |
| Body limits | `content-length` is refused before reading, and streamed bodies are cut off at the limit |
| Background work | `BackgroundTasks` is drained with the requests, inside the same budget; a task that throws is reported with its name rather than taking the isolate down |
| Failure taxonomy | 400, 401 with `WWW-Authenticate`, 403, 405 with `Allow`, 413, 415, 422, 500 opaque with the detail going to `onError` |
| Validation | a failed constraint answers 422 naming every field, whether it was caught by `ValidatedExtractable` or thrown by a generated `deserialize` |
| WebSockets | binary frames, subprotocol negotiation, close codes, broadcast between connections, and an upgrade served beside HTTP on one router |
| Rendering | templates loaded from a real directory, partials resolved, interpolation escaped |

Measured, on one developer machine, worst case over a 100-route table:
**27us per match**. That is a floor to compare against, not a promise.

## Missing, in the order it matters

### 1. Nothing has run for longer than a test

No soak test, no memory profile over hours, no leak check. A router built once
and read forever is the easy case, and the error sink is now scoped to a zone
rather than the process, so two routers no longer fight. What is untested is
hours of traffic rather than seconds of it.

### 2. Timeouts are opt-in, and cannot cancel

`RequestTimeout` bounds how long a handler may take and answers 503 past the
budget. What it cannot do is stop the handler: Dart has no cancellation, so the
work keeps running with nobody waiting for it. A handler that leaks a resource
on a slow path still leaks it. Nothing is installed by default either, so an
application that never adds the layer has no deadline at all.

### 3. Stateful middleware can be released, and nothing yet does

`DisposableLayer` gives a layer a `dispose()` that `close(drain:)` calls, after
background work has drained and whether or not there is any. A rate limiter
holding a client, a metrics flusher, a key cache on a timer: each now has
somewhere to release it, and one that throws does not stop the others.

None of the eight built-in layers implements it, because none owns anything. So
the mechanism exists and is tested, and no shipped code exercises it under real
load — the same distinction the top of this document draws about test coverage.

### 4. No backpressure, and no way to add it

tower's `Service` has two methods. `call` handles a request; `poll_ready` says
whether the service can take one at all. Everything tower builds on top of that
second method — `limit`, `load_shed`, `balance` — has no counterpart here.

Dart's equivalent of a service is a function. A function cannot decline before
it is called, so a Dust layer learns about a request only once it already has
it. Shedding load means answering 503 from inside the handler, after the work
of accepting and parsing is already done.

`RequestTimeout` bounds a slow request and is the closest thing available. It
is not the same thing: a timeout reacts after the fact, where backpressure
refuses in advance. Anyone arriving from tower will look for the second and
find only the first.

### 5. Multipart: buffered by default, streaming when you need it

`MultipartExtractable` holds every part in memory at once, bounded by the body
limit. That is the right trade for a form — several parts, readable in any
order — and a memory amplifier the moment the limit is raised for uploads.

`StreamedMultipartExtractable` is the answer for those: the parts arrive one at
a time and can be piped straight to disk, so nothing larger than a socket chunk
is ever held. The trade is explicit — the parts are ordered and consumable once,
so a handler needing to read part three before part one still has to buffer.

Two limits, because they stop different things. The **body** limit bounds the
whole request, checked up front against `content-length` and as the bytes flow
when there is none — a streamed upload usually declares no length. The **part**
limit bounds one file, and without it a single part can be the entire body
budget.

`example/multipart_stream.dart` shows it, including the part worth copying: the
file is stored under a generated id and the client's filename is kept as data,
because `../../etc/passwd` is a valid filename as far as a client is concerned.

### 6. Shutdown drains, but nothing tests it under load

`serve` returns a handle that counts in-flight requests, and `close`
waits for them within a deadline. It has never been exercised against real
traffic mid-deploy, which is the case that matters.

A cluster shutdown is bounded now: `test/serving/isolates_shutdown_test.dart`
wedges a worker isolate on a synchronous loop and shows `close` still returns
inside `drain + 5s` and kills the isolate. What that test also shows is worse
than it sounds — **killing the isolate does not release the socket it bound**,
so the port stays held until the process exits. A supervisor that restarts a
wedged worker in-process will not get the port back; replacing the process is
the only recovery.

### 7. Observability has traces and logs, and metrics are the application's

`RequestId` and `AccessLog` cover the log line, `onError` covers the failure,
and `Tracing` records one span per request in W3C Trace Context, so a trace
survives a hop into a service written in something else.

Metrics stay out of the runtime on purpose: what to count, how to bucket it, and
what to call it are decisions only the application can make, and a built-in set
that does not match your dashboard is worse than none. `AccessRecord` carries
what a counter needs, `matchedRoute` included — which is the part that decides
whether the labels aggregate or explode into one series per id.
`example/metrics.dart` builds a Prometheus endpoint on it.

No exporter ships either. `SpanExporter` is one method, deliberately, so
binding to a collector stays an application's dependency rather than the
runtime's.

### 8. No TLS guidance

`serve` is re-exported without a word about certificates, HTTP/2, or running
behind a proxy. Most deployments terminate TLS upstream; that should be said
rather than assumed.

### 9. The matcher is bucketed, not a trie

Routes bucket by their whole literal prefix: 29us at 100 routes, 13us at 1000,
11us for 500 sharing one prefix, 6us for static paths. What is left is a scan
within a bucket, so routes that are identical up to their first parameter still
compete. A trie would fix that; the numbers do not yet justify it.

### 10. One core unless you ask for more, and nothing says so

`serve` serves from the isolate that called it, and a Dart isolate is one
thread. On a four-core machine that is one core busy and three idle, under any
load, with nothing in the logs or the metrics to say the ceiling is the
process rather than the application.

`serveIsolates` fixes it — the same answer `uvicorn --workers` and `gunicorn -w`
give Python, for the same reason — but it is opt-in, and its own default is two
isolates rather than the core count. An application that never calls it is
correct, passes every test, and quietly wastes most of the machine.

Warning about it automatically is worse than it sounds: `Platform.numberOfProcessors`
reports the host's cores, not the container's CPU limit, so the obvious check
fires loudly and wrongly on every constrained deployment. Until there is a
reliable way to read the effective limit, this stays a documentation problem.

### 11. CI has a coverage gate, and no benchmark gate

The package is in the `dart-packages` matrix in `.github/workflows/ci.yml`, so
analyze and test run on every commit, and `scripts/dart/coverage.sh` fails the
build below a 100% floor. What is still missing is a benchmark regression
check, and the suite has never run on any machine but one.

### 12. Coverage is complete, and now gated

100% of lines, 1204 of 1204. Reaching it was worth more than the number: closing
the gaps, and then hunting past them, surfaced six real defects — each now
pinned by a test.

| Found | Was |
| :--- | :--- |
| `guard` swallowed `HijackException` | every WebSocket upgrade behind a verb builder answered 500 |
| `ws()` discarded the negotiated subprotocol | `session.protocol` was always `null` |
| A killed cluster isolate keeps its bound socket | an exclusive rebind of the port fails until the process exits |
| HTML mode cached `/` as immutable | the root resolves through the default document, so the shell was pinned for a year — the exact failure HTML mode exists to prevent |
| `coerce<int>` read `0x10` as 16 | `?id=0x10` and `?id=16` named the same record, and a check written against the text saw two different strings |
| A 401 challenge could carry CRLF | an extractor building its realm from anything a client influenced was a response-splitting hole |

The floor earned its keep the hour it landed: adding the request extension
left `multipart()` and `peer()` uncovered, and the gate caught both before the
change was finished.

A covered line is still not a checked line — 100% says every line ran, not that
every branch through it was asserted.

### 13. The API is unstable by design

The last few days moved `RouteGroup` to `Router`, `@Ctx` to `@State`, and
dropped OpenAPI. `0.1.0-beta.1` is published, so changes now cost a version
rather than a commit, but the surface is not settled and the generator that
will be its main consumer does not exist yet.

Two known changes are still owed, both of them axum parity:

- **`serve` should be `serve`.** axum calls it `serve` even though its
  second argument is a `Router`; the qualifier names the argument type the
  caller can already see. It exists only to dodge a collision with
  `shelf_io.serve` that cannot happen, since shelf is imported prefixed.
- **`Router` should implement a `Service` interface** rather than converting
  through `Router.handler`. A Dart class with a `call` method is assignable to
  a matching function type, so `Router implements Service` would be passable
  anywhere a shelf `Handler` is wanted, with no conversion — the same reason
  `axum::serve(listener, app)` takes a `Router` directly.

Argument order and the shutdown model should not follow axum. It takes a bound
listener because that is the Rust idiom, and gates shutdown on `Drop`; Dart
binds inside `serve` by convention and has no destructor, so `ServerHandle`
with a bounded `close(drain:)` is the better fit and keeps a timeout axum does
not have.

## What would make it ready

In order, each one gated on the previous:

1. Multipart streaming, so an upload limit is not a memory limit.
2. A teardown hook on `Layer`, so middleware may own a resource.
3. A soak test: sustained traffic for an hour, memory flat.
4. A deployment note covering TLS termination and proxies.
5. The plugin, so generated code is what gets exercised rather than
   hand-written stand-ins.
6. Only then a version number and a freeze on the public surface.

Until those are done, the honest description is "a runtime with good test
coverage of its own semantics", not "production ready".
