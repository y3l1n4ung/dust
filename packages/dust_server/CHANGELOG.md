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
- `serveCluster` is now `serveIsolates`, and `ServerCluster` is now
  `ServerIsolates`. "Cluster" reads as multiple machines; this forks isolates on
  one box, the way `uvicorn --workers` forks processes. Naming the isolate is
  also the clearest warning that state does not cross one.

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
