import 'dart:io';
import 'dart:isolate';

import 'package:dust_server/server.dart';

/// Using every core.
///
/// A Dart isolate runs on one core. `serveCluster` spawns several, all sharing
/// one listening socket, and the OS spreads connections across them — so a
/// four-core box serves roughly four times what one isolate does.
///
/// > **State does not cross isolates.** Each one runs the factory and gets its
/// > own copy of everything it builds. An in-memory cache becomes N caches with
/// > different contents; an in-memory session store means a user is signed in on
/// > one isolate and not the next; a counter counts a fraction of the traffic.
/// > Anything shared has to live outside the process — Redis, Postgres, SQLite in
/// > WAL mode.
///
/// That constraint is why the factory is a function rather than a router: a
/// `Router` cannot be sent across isolates, so each one has to build its own.
///
/// > **Measure from another process.** A load generator in the same process as
/// > the server competes with the main isolate for its event loop, and starves
/// > exactly the isolate it shares with. Measured that way this cluster looks
/// > badly skewed — main taking 2% of traffic — and measured from a separate
/// > process the same cluster is even: 18/17/15/10 across four isolates. The
/// > skew was the measurement, not the server. `wrk`, `curl`, or anything out of
/// > process gives the real answer.
///
/// One isolate is the right number more often than people expect. Isolates cost
/// memory, they make in-process state useless, and a server whose work is
/// waiting on a database is not CPU-bound in the first place. Measure before
/// reaching for this.
///
/// Run it with `dart run example/clustered_isolates.dart`:
///
/// ```bash
/// # Each request may land on a different isolate, so the counter jumps about
/// for i in $(seq 6); do curl -s localhost:8080/whoami; echo; done
/// ```
///
/// ```json
/// {"isolate":"main","seen":1}
/// {"isolate":"main","seen":1}
/// {"isolate":"main","seen":2}
/// ```
///
/// The `seen` counts do not add up to six, and that is the lesson rather than a
/// bug: each isolate is counting only what it handled.
Future<void> main() async {
  final cluster = await serveCluster(
    buildApp,
    InternetAddress.anyIPv4,
    8080,
    // One per core. More than that adds memory and context switching, not
    // throughput.
    isolates: Platform.numberOfProcessors,
  );
  stdout.writeln(
      'serving on 8080 across ${Platform.numberOfProcessors} isolates');

  await ProcessSignal.sigint.watch().first;
  // Drains every isolate, and gives up on one that will not finish rather than
  // hanging the shutdown forever.
  await cluster.close(drain: const Duration(seconds: 15));
}

/// Builds the application. Called **once per isolate**.
///
/// A top-level function, not a closure: what crosses to an isolate has to be
/// sendable, and a closure over local state is not.
Router buildApp() {
  return Router()
    ..route('/whoami', get(whoAmI))
    ..withState(IsolateCounter());
}

/// `GET /whoami`
Future<Map<String, Object?>> whoAmI(Request request) async {
  final counter = await request.state<IsolateCounter>();

  return {
    'isolate': Isolate.current.debugName ?? 'unnamed',
    'seen': ++counter.seen
  };
}

/// Per-isolate state, which is the only kind there is.
final class IsolateCounter {
  /// How many requests **this isolate** handled.
  int seen = 0;
}
