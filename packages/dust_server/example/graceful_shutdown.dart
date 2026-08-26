import 'dart:async';
import 'dart:io';

import 'package:dust_server/server.dart';

/// Shutting down without dropping requests in flight.
///
/// A process that exits the moment it is signalled drops every request it had
/// already accepted. Under a rolling deploy that is a burst of failures on every
/// release — and the requests it kills are the slow ones, which are the ones
/// most likely to have been half-way through a write.
///
/// `close(drain:)` stops accepting, then waits for what it accepted. It returns
/// **`false`** when the deadline passed with work still running, and that return
/// value is the point: it is your only chance to notice, and most code drops it.
///
/// Three things worth getting right:
///
/// * **`SIGTERM`, not just `SIGINT`.** Docker, Kubernetes, and systemd all send
///   `SIGTERM`. Watching only `SIGINT` means graceful shutdown works when you
///   press ctrl-C and never in production.
/// * **The drain budget must be under the platform's grace period.** Kubernetes
///   sends `SIGKILL` 30 seconds after `SIGTERM` by default, so a 60-second drain
///   is a 30-second drain followed by a hard kill.
/// * **Dart cannot cancel a handler.** Draining waits; it does not stop
///   anything. A request still running when the budget expires keeps running
///   until the process dies underneath it.
///
/// Run it with `dart run example/graceful_shutdown.dart`, then in another
/// terminal:
///
/// ```bash
/// curl -s localhost:8080/slow &   # takes two seconds
/// sleep 0.2
/// kill -TERM $(pgrep -f 'example/graceful_shutdown.dart')
/// # the curl finishes; the server exits after it
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  // Both signals, so it behaves the same under a container runtime and under a
  // keyboard. SIGTERM is the one that matters.
  await Future.any([
    ProcessSignal.sigterm.watch().first,
    ProcessSignal.sigint.watch().first,
  ]);
  stdout.writeln('draining ${server.inFlight} request(s)');

  final settled = await server.close(drain: const Duration(seconds: 15));
  if (!settled) {
    // Worth logging loudly. It means requests were abandoned, and the only way
    // to know is this return value.
    stderr.writeln(
        'shutdown deadline passed with ${server.inFlight} still running');
    exitCode = 1;
  }
  stdout.writeln('stopped');
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  return Router()
    ..route('/quick', get(quick))
    ..route('/slow', get(slow));
}

/// `GET /quick`
Map<String, Object?> quick(Request request) => const {'ok': true};

/// `GET /slow` — long enough to still be running when a signal arrives.
Future<Map<String, Object?>> slow(Request request) async {
  await Future<void>.delayed(const Duration(seconds: 2));

  return const {'ok': true};
}
