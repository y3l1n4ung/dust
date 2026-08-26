# Serving

## One isolate

```dart
final server = await serveRouter(app, InternetAddress.anyIPv4, 8080);
stdout.writeln('listening on ${server.address.host}:${server.port}');
```

`ServerHandle` reports `address`, `port`, and `inFlight`. The address and port
are captured at bind time, so they stay readable after the server closes.

## Shutting down

```dart
await ProcessSignal.sigterm.watch().first;

if (!await server.close(drain: const Duration(seconds: 15))) {
  log.warning('exiting with ${server.inFlight} requests still running');
}
```

`close` stops accepting, then waits for requests already accepted. It returns
`false` when the deadline passed with work still in flight, leaving the decision
to wait longer or exit to the caller.

Requests still queued in a client have not been accepted, and shutdown owes them
nothing; they see a refused connection.

## Several isolates

Dart runs one isolate on one thread, so one server uses one core, however many
the machine has. Isolates do not share memory, so reaching the other cores
means running the server several times over rather than adding threads to it.

> [!IMPORTANT]
> `serveRouter` alone uses **one** core. On a four-core machine that is three
> quarters of the hardware idle, under any load, with nothing in the logs to
> say so. Nothing warns you, and the throughput ceiling looks like the
> application's rather than the process's.

This is the model `uvicorn --workers` and `gunicorn -w` solve for Python, for
the same reason: one interpreter holds the GIL, one isolate holds a thread, so
the runtime cannot spread one server across cores. Both answers are the same —
run it N times behind a shared socket. Dart's isolates are lighter than worker
processes, sharing a VM rather than an OS process, but the arithmetic is
identical.

A threaded runtime needs none of it. axum on tokio, or a Go server, schedules
across every core from one process, so there is nothing to cluster.

Sockets bound with `shared: true` let several isolates accept on one port, and
the operating system balances between them.

```dart
Router buildApp() => Router()..route('/', get(home));

final cluster = await serveCluster(
  buildApp,
  InternetAddress.anyIPv4,
  8080,
  isolates: Platform.numberOfProcessors,
);
```

The factory has to be a top-level or static function, since it is sent to each
isolate. **State does not cross isolates**: every isolate builds its own router
and its own copy of whatever the factory builds. Anything genuinely shared, a
cache or a counter, belongs outside the process.

`cluster.close(drain:)` drains every isolate before killing it.

## TLS

`serveRouter` takes a `SecurityContext`:

```dart
final context = SecurityContext()
  ..useCertificateChain('fullchain.pem')
  ..usePrivateKey('privkey.pem');

await serveRouter(app, InternetAddress.anyIPv4, 443, securityContext: context);
```

Most deployments terminate TLS upstream instead, at a load balancer or a reverse
proxy. When they do, the proxy's `X-Forwarded-*` headers are ordinary headers
here: nothing reads them automatically, and nothing should, since trusting them
without knowing the proxy is a spoofing hole.

## Trying it

`dart run example/graceful_shutdown.dart`, then from another terminal:

```bash
curl -s localhost:8080/slow &   # takes two seconds
sleep 0.2
kill -TERM $(pgrep -f 'example/graceful_shutdown.dart')
```

The `curl` finishes with a 200 and the process exits after it. Three things in
that example are worth copying rather than rediscovering:

* **Watch `SIGTERM`, not only `SIGINT`.** Docker, Kubernetes, and systemd all
  send `SIGTERM`. A server that watches only `SIGINT` drains when you press
  ctrl-C and never in production.
* **Keep the drain budget under the platform's grace period.** Kubernetes sends
  `SIGKILL` 30 seconds after `SIGTERM` by default, so a 60-second drain is a
  30-second drain followed by a hard kill.
* **Check what `close` returns.** `false` means the deadline passed with work
  still running — requests were abandoned. It is the only signal you get, and it
  is the one most code throws away.

Draining waits; it does not cancel. Dart has no cancellation, so a request still
running when the budget expires keeps running until the process dies underneath
it.

### Work that outlives the response

`close(drain:)` counts **requests**. Anything spawned outside one —
`unawaited(sendReceipt(order))` — is invisible to it, so every deploy kills that
work mid-flight with nothing logged. A customer gets their 201 and never gets
their email.

`BackgroundTasks` closes that hole. Pass one to `serveRouter` and it is drained
alongside the requests, inside the same budget:

```dart
final tasks = BackgroundTasks();
final server = await serveRouter(app, address, 8080, background: tasks);

// in a handler
final tasks = await request.state<BackgroundTasks>();
tasks.run('receipt', () => mail.sendReceipt(order));
```

`run` returns `false` when the registry is draining, which is worth acting on:
the order was placed and the receipt will not be sent, so it belongs in an outbox
rather than lost. A task that throws is reported through `onError` with its name
attached rather than raised, since an unhandled asynchronous error would take the
isolate down.

> **In-process and unpersisted.** A task lost to a crash is gone and nothing
> retries it. Right for work that is *nice* to finish, wrong for work that
> *must* happen — that needs an outbox table or a real queue, and this is not a
> substitute for one.

`dart run example/background_tasks.dart`:

```bash
curl -s -X POST localhost:8080/orders -H 'content-type: application/json' \
  -d '{"email":"ada@example.com"}'
curl -s localhost:8080/receipts
```

```json
{"placed":true,"receiptQueued":true}
{"sent":[]}
```

The receipt is empty because the handler returned before the task ran — which is
the point. `server.pendingTasks` reads 1 at that moment, and `close` waits for it.

Using every core is `dart run example/clustered_isolates.dart`:

```bash
for i in $(seq 6); do curl -s localhost:8080/whoami; echo; done
```

```json
{"isolate":"main","seen":1}
{"isolate":"main","seen":1}
{"isolate":"main","seen":2}
```

The counts do not add up to six, and that is the lesson rather than a bug: state
does not cross isolates, so each one counts only what it handled. An in-memory
cache becomes N caches with different contents, and an in-memory session store
signs a user in on one isolate and not the next. Anything shared has to live
outside the process.

TLS is `dart run example/tls.dart <cert.pem> <key.pem>`, and mostly you should
not: a proxy in front already terminates it and renews the certificate, and doing
it in two places means one of them will lapse.


