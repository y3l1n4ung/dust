# Server Runtime: Production Readiness

Status: `packages/dust_server` is **not** production ready. This document says
what is covered, what is missing, and what would have to be true before an
application is put in front of real traffic.

For the feature itself, see the [Server Plugin Design](server-plugin.md).

## Where it stands

834 tests at 100% line coverage, no analyzer findings, every file under 180
lines. That says the code does what its tests say. It does not say the tests
ask the right questions, which is what the rest of this document is about.

## Covered

Router coverage is organized one folder per case under `test/router`, so a gap
is visible as a thin folder rather than hidden inside a long file:

| Folder | Cases |
| :--- | :--- |
| `matching/` | placeholders, percent encoding, literal segments, trailing slashes |
| `methods/` | HEAD fallback, 405 and `Allow`, every verb builder |
| `composition/` | nest, merge, duplicates, sealing, fallback, describe |
| `middleware/` | layer order down the tree, layer types |
| `state/` | scoping and override, missing state |
| `lifecycle/` | body limit, error reporting |
| `paths/` | prefix normalization and joining |
| `conformance/` | parity with `shelf_router`, hand-picked and randomized |
| `integration/` | real socket, `shelf` interop, concurrency, throughput |
| `hardening/` | pathological input, untrusted request fields |

Beyond the router, `test/example/` serves both example applications over a
loopback socket and drives them with a real client: the CRUD flow, every
failure code, the layers, shutdown, and — for the chat example — rendered
HTML, JSON, and a WebSocket upgrade answered by one route table.

| Area | Evidence |
| :--- | :--- |
| Route matching | agrees with `shelf_router` on 24 hand-picked paths and 1000 generated ones |
| Real HTTP | serves over a socket and is driven by a real client: methods, status, path capture, percent-decoding, 404, 405, HEAD |
| Ecosystem fit | runs behind `shelf` middleware, hosts plain `shelf` handlers, mounts inside `shelf_router`, composes with `Cascade` |
| Concurrency | 200 simultaneous requests keep their own path parameters; the matcher is immutable after build |
| Pathological input | 10000-segment paths, 100 KB segments, and a 200-route table all answer in bounded time |
| Untrusted fields | a method carrying CRLF cannot forge a response header; paths are matched literally after Dart normalizes them |
| Body limits | `content-length` is refused before reading, and streamed bodies are cut off at the limit |
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

### 3. Multipart buffers the whole body

`MultipartExtractable` holds every part in memory at once, bounded only by the
body limit. A 1 MiB default makes that survivable; anything raising the limit
for uploads makes it a memory amplifier. Streaming parts is the fix.

### 4. Shutdown drains, but nothing tests it under load

`serveRouter` returns a handle that counts in-flight requests, and `close`
waits for them within a deadline. It has never been exercised against real
traffic mid-deploy, which is the case that matters.

A cluster shutdown is bounded now: `test/serving/cluster_shutdown_test.dart`
wedges a worker isolate on a synchronous loop and shows `close` still returns
inside `drain + 5s` and kills the isolate. What that test also shows is worse
than it sounds — **killing the isolate does not release the socket it bound**,
so the port stays held until the process exits. A supervisor that restarts a
wedged worker in-process will not get the port back; replacing the process is
the only recovery.

### 5. Observability has traces and logs, and no metrics

`RequestId` and `AccessLog` cover the log line, `onError` covers the failure,
and `Tracing` records one span per request in W3C Trace Context, so a trace
survives a hop into a service written in something else. What is still missing
is metrics: no counters, no histograms, nothing that answers "how many" or
"how slow" without reading every span.

No exporter ships either. `SpanExporter` is one method, deliberately, so
binding to a collector stays an application's dependency rather than the
runtime's.

### 6. No TLS guidance

`serve` is re-exported without a word about certificates, HTTP/2, or running
behind a proxy. Most deployments terminate TLS upstream; that should be said
rather than assumed.

### 7. The matcher is bucketed, not a trie

Routes bucket by their whole literal prefix: 29us at 100 routes, 13us at 1000,
11us for 500 sharing one prefix, 6us for static paths. What is left is a scan
within a bucket, so routes that are identical up to their first parameter still
compete. A trie would fix that; the numbers do not yet justify it.

### 8. CI has a coverage gate, and no benchmark gate

The package is in the `dart-packages` matrix in `.github/workflows/ci.yml`, so
analyze and test run on every commit, and `scripts/dart/coverage.sh` fails the
build below a 100% floor. What is still missing is a benchmark regression
check, and the suite has never run on any machine but one.

### 9. Coverage is complete, and now gated

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

### 10. The API is unstable by design

`publish_to: none`, and the last few days moved `RouteGroup` to `Router`,
`@Ctx` to `@State`, and dropped OpenAPI. None of that is settled enough to
promise compatibility, and the generator that will be its main consumer does
not exist yet.

## What would make it ready

In order, each one gated on the previous:

1. Multipart streaming, so an upload limit is not a memory limit.
2. A soak test: sustained traffic for an hour, memory flat.
3. A deployment note covering TLS termination and proxies.
4. The plugin, so generated code is what gets exercised rather than
   hand-written stand-ins.
5. Only then a version number and a freeze on the public surface.

Until those are done, the honest description is "a runtime with good test
coverage of its own semantics", not "production ready".
