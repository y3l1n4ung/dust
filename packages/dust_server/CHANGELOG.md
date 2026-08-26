# Changelog

All notable changes to `dust_server` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.1.0-beta.2] - 2026-08-25

Naming, brought in line with axum. All three are breaking, with no deprecated
forwarders — the package is one release old and the names are wrong now rather
than later. See [axum parity](https://github.com/y3l1n4ung/dust/blob/main/docs/design/server-axum-parity.md).

### Changed

- `serveRouter` is now `serve`. The old name described its argument's type,
  which the caller can already see.
- `serve` takes an `InternetAddress` rather than an `Object`. `HttpServer.bind`
  accepts a `String` host, which buys a hidden DNS lookup and a typo that fails
  at runtime; `InternetAddress.anyIPv4` and `.loopbackIPv4` say what `0.0.0.0`
  and `127.0.0.1` mean and cannot be mistyped. `serveIsolates` already took one.
- `serveIsolates` requires `isolates:`. The old default of two was neither
  "one, explicitly" nor "use the machine".
- `serveCluster` is now `serveIsolates`, and `ServerCluster` is now
  `ServerIsolates`. "Cluster" reads as multiple machines; this forks isolates on
  one box, the way `uvicorn --workers` forks processes. Naming the isolate is
  also the clearest warning that state does not cross one.

### Added

- `DisposableLayer`, a `Layer` that also declares `dispose()`. Shutdown walks
  the router — nested groups and `routeLayer` included — and releases each one
  after background work has drained. A `dispose` that throws does not stop the
  others. Separate from `Layer` because Dart's `implements` requires every
  member re-declared, so adding a method there would break every existing layer.
- `ServerIsolates.alive` and an `onIsolateError` callback on `serveIsolates`.
  A dead isolate was invisible: the port stays bound by the survivors, so
  traffic kept flowing at reduced capacity with nothing to say so. It still
  cannot be restarted — killing an isolate does not release its socket, so a
  replacement cannot rebind the port — but the loss is no longer silent.
- `Service`, with `Router` implementing it. Dart tears off `call` implicitly, so
  a router is assignable to a shelf `Handler` with no conversion — the same
  reason `axum::serve(listener, app)` takes a `Router` directly.

### Examples

- `disposable_layers.dart` — releasing what a layer owns, on shutdown.
- `router_as_handler.dart` — handing a router to `shelf` middleware, since
  `Router` implements `Service` and is therefore already a `Handler`.
- `isolate_failure.dart` — knowing when a serving isolate dies, and why it
  cannot be replaced.

### Fixed

- A response header carrying a control character now answers 500 and reports
  through `onError`. `dart:io` refuses to write one — which is what stops a
  response being split — but it threw inside `shelf_io` after the handler had
  returned, outside every catch in the package. The request never completed,
  the client waited until it timed out, and nothing in the log said which
  handler did it: a leaked connection and an invisible failure. Tab is still
  allowed; every other control character is refused.
- A `DisposableLayer` used on more than one router is disposed once, by
  identity, rather than once per registration. A layer applied at the root and
  again on a subtree owns one resource, and closing it twice failed silently
  because the guard around `dispose` swallows the second error. Deduplication
  is by identity, not equality: two separate instances that compare equal each
  own their own resource, and collapsing them would leak one.
- `serveIsolates` no longer hangs forever when the router factory throws
  inside a spawned isolate. It waited on a port the isolate writes to only
  after the factory has already succeeded, so a factory that works in the
  parent and fails in the child — a locked file, an environment variable read
  per isolate, anything the parent already holds — left the server never
  started and nothing logged. It now fails with the isolate named and the
  original error attached, and cleans up: no port left bound, no isolate left
  running.
- Numeric coercion no longer accepts surrounding whitespace. Dart's parsers
  trim before parsing, so `?id=%2010` and `?id=10` produced the same number
  from two different requests, and anything keyed on the raw text — a cache
  key, a rate-limit bucket, a dedup check — disagreed with the handler about
  which request it had. Applies to `int`, `double`, `num`, and `BigInt`. This
  is the same class as the `0x` prefix already rejected, which the earlier fix
  did not cover.

### Removed

- `package:dust_server/router.dart` no longer re-exports shelf's `serve`. It
  occupied the name the package needed for its own entry point. Import
  `package:shelf/shelf_io.dart` directly if you were using it.

## [0.1.0-beta.1] - 2026-08-24

First beta, and the first release of this package. The runtime is complete
enough to build on; the API may still change before 1.0, so pin the exact
version.

### Added

Routing in the shape axum uses — `route`, `nest`, `merge`, `mount`, `layer`,
`routeLayer`, `withState`, `fallback` — over `shelf`, with its own matcher.

- **Extraction**: path, query, header, host, cookie, state, JSON, form,
  multipart (buffered and streaming), raw and streamed bodies, bearer tokens,
  Basic credentials, API keys, session ids, and `firstOf` to compose them.
  `valid`, `optional`, and `fallible` wrap any of them.
- **Responses**: typed dispatch from what a handler returns, `Rejection` with a
  failure taxonomy, redirects, server-sent events, streamed bodies, templates.
- **Layers**: CORS, compression, request id, access log, path normalization,
  security headers, request timeout.
- **Serving**: graceful shutdown that drains requests and background work, TLS,
  isolate clustering, static files with single-page support.
- **Observability**: W3C Trace Context spans, an access record carrying the
  matched route, and `onError` for failures.

51 examples in `example/`, one question each, all served over a real socket by
`test/example/`.

Pinned at this release: **1275 tests, 1595/1595 lines, 100% line coverage.**
Prose elsewhere says "over 1,200" on purpose — an exact figure in a document
nobody recounts goes stale on the next commit, and did so four times before this
one.

### Known limits

- The code generator this runtime exists for does not exist yet. Everything here
  is written by hand today.
- No metrics, sessions, or rate limiting in the runtime. Each is policy, and each
  ships as an example instead.
- Range requests work for static files, not for a dynamic body.
- `serveIsolates` gives each isolate its own state; anything shared belongs
  outside the process.

### Seventeen defects found and fixed before this beta

Fifteen were found by writing an example or probing a combination of layers,
two by reading the code. Three were in code written the same day. The ones worth
knowing about, because each was silent:

| Area | What was wrong |
| :--- | :--- |
| Server-sent events | never streamed — every event was held until the stream ended |
| Streamed responses | a `Stream` return answered 500; a hand-built one buffered |
| `mount('/')` | claimed only the bare root, so every deep link in a single-page build 404'd |
| Route order | a router's own routes were flattened ahead of its children whatever the declaration order |
| Nested `layer` | ran only after a route matched, so `NormalizePath` inside a `nest` did nothing |
| Body limits | a router limit *loosened* a stricter per-route one |
| WebSocket upgrades | traced as errors and missing from the access log |
| Background work | not drained by shutdown, and inheriting a span that had already ended |
| Coercion | `?id=0x10` and `?id=16` were the same request |
| 401 challenges | accepted CRLF, which is response splitting |
