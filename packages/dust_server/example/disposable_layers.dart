import 'dart:async';
import 'dart:io';

import 'package:dust_server/server.dart';

/// Releasing what a layer owns, when the server stops.
///
/// A `Layer` is a class, so it can hold configuration — and that is where
/// production middleware ends up holding a *resource* too: a client for a rate
/// limiter, a flusher for metrics, a timer refreshing signing keys. `Layer`
/// declares only `toMiddleware()`, so there is nowhere to close any of it.
///
/// `DisposableLayer` adds `dispose()`. `close(drain:)` walks the router —
/// nested groups and `routeLayer` included — and calls it on everything it
/// finds, **after** in-flight requests and background work have drained, since
/// either may still be using what the layer owns.
///
/// axum needs no equivalent: Rust drops a layer when its router goes. Dart has
/// no destructor, so without this a long-running process leaks every resource
/// its middleware acquired.
///
/// Two details worth knowing. A `dispose` that throws does not stop the others
/// — shutdown has already stopped accepting, and losing the rest of the
/// teardown to one bad release is worse than the error. And it is a separate
/// interface from `Layer` because Dart's `implements` requires every member to
/// be re-declared, so adding a method to `Layer` would break every layer that
/// already exists.
///
/// Run it with `dart run example/disposable_layers.dart`:
///
/// ```bash
/// curl localhost:8080/         # counted
/// # then Ctrl-C, and watch the counter get flushed exactly once
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  return Router()
    ..layer(RequestCounter(stdout.writeln))
    ..route('/', get((request) async => 'counted'));
}

/// Counts requests and flushes the total once, on shutdown.
///
/// Standing in for anything that batches: a metrics exporter, a buffered audit
/// log, a writer that would rather send one row than a thousand.
final class RequestCounter implements DisposableLayer {
  /// Reports the total through [report] when the server closes.
  RequestCounter(this.report);

  /// Where the total goes.
  final void Function(String line) report;

  int _seen = 0;

  @override
  Middleware toMiddleware() {
    return (inner) => (request) {
          _seen++;
          return inner(request);
        };
  }

  @override
  Future<void> dispose() async => report('flushed $_seen requests');
}
