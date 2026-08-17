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

Dart runs one isolate on one thread, so one server uses one core. Sockets bound
with `shared: true` let several isolates accept on one port, and the operating
system balances between them.

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

```bash
dart run example/todo_api.dart
```

```bash
# the server answers while it is up
curl -s localhost:8080/health

# ask it to stop, and watch it drain rather than cut connections
kill -INT $(pgrep -f 'example/todo_api.dart')
```

The process prints how many requests were still in flight and waits for them
within the deadline before exiting.
