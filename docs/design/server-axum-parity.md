# Server Runtime: axum Parity

Status: settled, with the naming changes landed in `0.1.0-beta.2`.

`dust_server` is a Dart HTTP runtime on `shelf` whose API is modelled on Rust's
axum. It is **not** built on axum — no Rust is involved — and the distinction
matters, because it decides which differences are bugs and which are correct.

This document records what matches, what deliberately does not, and what cannot.
It exists so those three categories stop being re-argued.

Everything here was read from source, not recalled: `axum-0.8.9`,
`axum-core-0.5.6`, `tower-service-0.3.3`, `tower-layer-0.3.3`, and
`tower-http-0.6.11` in the local cargo registry, against
`packages/dust_server` in this repository.

## What parity means here

Copy axum's **vocabulary and shapes**, because they are good and because a
reader who knows one should be able to read the other.

Do not copy axum's **mechanisms**, where they exist to satisfy Rust. Ownership,
`Drop`, and trait dispatch shape a lot of axum's surface, and reproducing them
in a language with garbage collection and no destructors produces worse code,
not more faithful code.

Three tests decide any given case:

1. Does the difference make the API harder to learn for someone who knows axum?
   Then close it.
2. Does closing it require Dart to pretend it has a Rust feature? Then do not.
3. Is the axum version simply weaker here? Then diverge and say so.

## Serving

```rust
// axum
pub fn serve<L, M, S>(listener: L, make_service: M) -> Serve<L, M, S>
```

```dart
// dust_server
Future<ServerHandle> serve(Router app, Object address, int port, {...});
```

| axum | dust_server | Verdict |
| :--- | :--- | :--- |
| `serve` | `serve` | matched |
| `serve(listener, app)` | `serve(app, address, port)` | deliberate |
| `Serve::with_graceful_shutdown(fut)` | `ServerHandle.close(drain: Duration)` | deliberate |
| `Serve::local_addr()` | `ServerHandle.port` | matched in effect |
| — | `ServerHandle.inFlight`, `.pendingTasks` | addition |

**The name is now `serve`.** It was `serveRouter`, which named the argument's
type — something the caller can already see. Taking the name required dropping
`export 'package:shelf/shelf_io.dart' show serve;` from `lib/router.dart`, which
had deliberately occupied it. Anyone who wants shelf's `serve` imports
`shelf_io` directly; the package's own entry point should own the name in the
package's own namespace.

**Argument order stays.** axum takes an already-bound listener because Rust
wants the socket owned explicitly. Dart's convention is `HttpServer.bind`, and
binding inside `serve` keeps the common case one call. The cost is a weakly
typed `Object address`, inherited from `shelf_io.serve` and ultimately from
`HttpServer.bind`, which accepts a `String` host or an `InternetAddress`.

A listener-first `serve(HttpServer listener, Router app)` would delete that
`Object` and every parameter forwarded to `bind`. It was considered and left
alone: it makes the simple case two statements, and `ServerHandle` — which
carries `port`, `inFlight`, and `pendingTasks` — is more useful than what axum
returns.

**Shutdown stays.** axum's `with_graceful_shutdown` takes a future and then
waits, unbounded, for in-flight connections. Dust returns a handle and bounds
the wait. Rust can rely on `Drop`; Dart has no destructor, so a handle is the
only shape that can be closed at all — and the timeout is a capability axum
lacks, not a compromise.

## Router

Full parity on everything that has a Dart meaning.

| axum | dust_server |
| :--- | :--- |
| `route` | `route` |
| `nest` | `nest` |
| `merge` | `merge` |
| `layer` | `layer` |
| `route_layer` | `routeLayer` |
| `fallback` | `fallback` |
| `with_state` | `withState` |
| `route_service`, `nest_service`, `fallback_service` | — |
| `method_not_allowed_fallback`, `reset_fallback` | — |
| `into_make_service`, `into_service`, `as_service`, `has_routes` | — |
| — | `mount` |

The `*_service` variants exist because axum distinguishes a handler from a
tower `Service`. Dart has one callable shape, so the distinction has nothing to
express. The `into_*` family is tower plumbing.

`mount` has no axum counterpart: it serves a subtree, which axum expresses with
`nest_service`.

### Verbs

axum generates `get post put patch delete head options trace connect` from a
macro, plus `any`. dust_server declares the same ten by hand in
`method_router.dart`. Exact match.

## Extraction

axum's split is `FromRequestParts` for anything readable from the head, and
`FromRequest` for anything that consumes the body — at most one, and last.
dust_server uses the same two names and the same rule, and enforces the
ordering as a build diagnostic rather than a trait bound.

Rejections carry the same status codes: 400, 401 with `WWW-Authenticate`, 403,
405 with `Allow`, 413, 415, 422.

`Extension<T>` and `fromExtractor` mirror axum's `Extension<T>` and
`middleware::from_extractor`.

## Responses

axum's rule is that a handler returns anything implementing `IntoResponse`, and
that a `String` becomes `text/plain` while a `Json<T>` becomes JSON.

dust_server keeps `IntoResponse` and the `String` rule, and makes JSON the
default for anything else rather than requiring a wrapper, because Dart has no
orphan-rule problem to work around and the wrapper buys nothing.

## Middleware

```rust
pub trait Layer<S> {
    type Service;
    fn layer(&self, inner: S) -> Self::Service;
}
```

```dart
abstract interface class Layer {
  Middleware toMiddleware();
}
```

Same concept, one extra hop: tower's `layer` wraps a service directly, where
dust's `toMiddleware` returns a wrapper that then takes the handler. Since
shelf's `Middleware` is `Handler Function(Handler)`, the two are the same
function curried differently.

`Router.layer` accepts either a `Layer` or a bare shelf `Middleware`, so a
stateful, configurable layer is a class and a one-off is a closure.

The built-in catalogue tracks `tower-http`, which is middleware only and has no
`serve` of its own: `cors`, `compression`, `trace`, `timeout`, `auth`, `limit`,
`set_header`. dust_server ships `Cors`, `Compression`, `AccessLog`,
`RequestTimeout`, `RequestId`, `NormalizePath`, `SecurityHeaders`, `Tracing`.

### Gap: no teardown

`Layer` declares one method. A layer holding a Redis client, a metrics flusher,
or a JWKS cache on a timer has nowhere to release it, and `ServerHandle.close`
does not look for one.

axum has the same interface and does not need the method, because Rust drops the
layer when its router goes. This is the clearest case in the document of a
difference that is a **Dart problem rather than an axum one**, and it wants a
separate `DisposableLayer` interface — adding a method to `Layer`, even with a
default body, fails analysis on all thirteen implementors, because Dart's
`implements` requires every member to be re-declared.

## Isolates

`serveIsolates(factory, address, port, isolates: N)` has **no axum counterpart**,
and should not be made to look like it has one.

axum on tokio schedules across every core from one process. Dart runs one
isolate on one thread and isolates share no memory, so reaching the other cores
means running the server several times over. The right reference is
`uvicorn --workers` or `gunicorn -w`, which solve the same problem for the same
reason — one interpreter holds the GIL, one isolate holds a thread.

It was `serveCluster`. "Cluster" now reads as multiple machines — Kubernetes,
Redis, database — so the name suggested joining or serving a cluster rather than
forking workers on one box. `serveIsolates` names the mechanism, which is right
here because the mechanism carries the surprising constraint: **state does not
cross isolates**, so a cache, counter, or session in a variable silently breaks
the moment `isolates > 1`.

Folding it into `serve(factory, ..., isolates: N)` was considered. It would
remove the footgun of forgetting it — one core of four, with nothing in the logs
to say so — but it forces every caller to pass a top-level factory even for the
single-isolate case, and 58 of the 100 existing call sites build their router
inline in a way a sendable factory cannot express.

## What cannot port

```rust
pub trait Service<Request> {
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    fn call(&mut self, req: Request) -> Self::Future;
}
```

`call` maps exactly onto shelf's `Handler`. `poll_ready` has no counterpart and
cannot get one: a Dart handler is a function, and a function cannot decline
before it is called.

Everything tower builds on that second method — `limit`, `load_shed`, `balance`
— is therefore unavailable. Shedding load in Dust means answering 503 from
inside the handler, after the work of accepting and parsing is already done.
`RequestTimeout` is the nearest thing and is not the same thing: a timeout
reacts after the fact where backpressure refuses in advance.

Anyone arriving from tower will look for the second and find only the first.

## Naming decisions

| Decision | Outcome |
| :--- | :--- |
| `serveRouter` | → `serve`, dropping the shelf `serve` re-export |
| `serveCluster` | → `serveIsolates` |
| `ServerCluster` | → `ServerIsolates` |
| `ServerHandle` | kept — no axum equivalent, and more useful than `Serve` |
| `Object address`, `int port` | kept — `HttpServer.bind` is the Dart convention |
| `mount` | kept — no good axum name for it |
| `toMiddleware()` | kept for now; `layer(Service)` would be closer |

## Open

- **`Layer` teardown**, as a `DisposableLayer` interface drained by
  `ServerHandle.close`.
- **`Router implements Service`**, so a router is passable wherever a shelf
  `Handler` is wanted with no conversion. A Dart class with a `call` method
  tears off to a matching function type implicitly, so this costs nothing.
- **`serveIsolates` default of two**, which is neither "one, explicitly" nor
  "use the machine". Defaulting to the core count has its own problem:
  `Platform.numberOfProcessors` reports the host's cores rather than the
  container's CPU limit.

## Rejected

- **Listener-first `serve(listener, app)`.** Closer to axum, deletes the
  `Object address` parameter, and makes the common case two statements while
  discarding a more useful handle.
- **Folding isolates into `serve`.** Kills a real footgun at the cost of a
  factory argument in every call, including the ones that never cluster.
- **A dedicated supervisor isolate**, uvicorn-style. Costs a core to supervise,
  which is the wrong trade on a two-core container; `onExit` on the main
  isolate's event loop does the same job for nothing.
- **`serveWorkers`.** Familiar from gunicorn and uvicorn, but "worker" implies
  a process, and hiding the isolate boundary hides the constraint that matters.
