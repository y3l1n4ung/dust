import 'dart:async';
import 'dart:io';
import 'dart:isolate';

import '../router/router_base.dart';
import 'graceful.dart';

/// Builds the application for one isolate.
///
/// Every isolate builds its own router, because a `Router` holds handlers and
/// state that cannot cross an isolate boundary. This has to be a top-level or
/// static function so it can be sent to a new isolate.
typedef RouterFactory = Router Function();

/// A cluster of isolates all serving one port.
///
/// Each isolate is a separate heap with its own router. This is not a thread
/// pool, and nothing is shared between them; see [serveCluster].
final class ServerCluster {
  ServerCluster._(this._isolates, this._handles, this.address, this.port);

  final List<Isolate> _isolates;
  final List<SendPort> _handles;

  /// The address every isolate bound to.
  final InternetAddress address;

  /// The port every isolate shares.
  final int port;

  /// How many isolates are serving.
  int get size => _isolates.length + 1;

  /// Stops every isolate, draining what each has in flight.
  Future<void> close({Duration drain = const Duration(seconds: 30)}) async {
    final stopped = <Future<void>>[];
    for (final handle in _handles) {
      final reply = ReceivePort();
      handle.send([drain.inMilliseconds, reply.sendPort]);
      stopped.add(reply.first.then((_) => reply.close()));
    }

    await Future.wait(stopped).timeout(
      drain + const Duration(seconds: 5),
      onTimeout: () => const <void>[],
    );
    for (final isolate in _isolates) {
      isolate.kill(priority: Isolate.immediate);
    }
  }
}

/// Serves [factory] from [isolates] isolates, all sharing one port.
///
/// Dart runs one isolate on one thread, so a single server uses one core, and
/// isolates do not share memory. Using the rest of the machine means running
/// the server several times over rather than adding threads to it.
///
/// [serveRouter] on its own uses one core. On a four-core machine that leaves
/// three idle under any load, and nothing reports it — the throughput ceiling
/// looks like the application's rather than the process's.
///
/// This is what `uvicorn --workers` and `gunicorn -w` do for Python, for the
/// same reason: the runtime cannot spread one server across cores, so it is run
/// several times behind a shared socket. A threaded runtime needs none of it,
/// so there is no equivalent to copy; this is here for the isolate model, not
/// as a convenience on top of [serveRouter].
///
/// The operating system load-balances accepted connections across sockets bound
/// with `shared: true`, which is what lets several isolates answer one port.
///
/// ```dart
/// Router buildApp() => Router()..route('/', get(home));
///
/// void main() async {
///   final cluster = await serveCluster(
///     buildApp,
///     InternetAddress.anyIPv4,
///     8080,
///     isolates: Platform.numberOfProcessors,
///   );
/// }
/// ```
///
/// State does not cross isolates. Anything shared, a cache or a counter, has to
/// live outside the process; each isolate gets its own copy of whatever
/// [factory] builds.
Future<ServerCluster> serveCluster(
  RouterFactory factory,
  InternetAddress address,
  int port, {
  int isolates = 2,
}) async {
  if (isolates < 1) {
    throw ArgumentError.value(isolates, 'isolates', 'must be at least one');
  }

  final local = await serveRouter(
    factory(),
    address,
    port,
    shared: true,
  );

  final spawned = <Isolate>[];
  final handles = <SendPort>[];

  for (var i = 1; i < isolates; i++) {
    final ready = ReceivePort();
    final isolate = await Isolate.spawn(
      _serveInIsolate,
      _ClusterSeed(factory, address, local.port, ready.sendPort),
      debugName: 'dust_server worker $i',
    );

    handles.add(await ready.first as SendPort);
    ready.close();
    spawned.add(isolate);
  }

  handles.add(_localHandle(local));
  return ServerCluster._(spawned, handles, address, local.port);
}

SendPort _localHandle(ServerHandle server) {
  final port = ReceivePort();
  port.listen((message) async {
    final request = message as List<Object?>;
    await server.close(drain: Duration(milliseconds: request[0]! as int));
    (request[1]! as SendPort).send(null);
    port.close();
  });
  return port.sendPort;
}

final class _ClusterSeed {
  const _ClusterSeed(this.factory, this.address, this.port, this.ready);

  final RouterFactory factory;
  final InternetAddress address;
  final int port;
  final SendPort ready;
}

Future<void> _serveInIsolate(_ClusterSeed seed) async {
  final server = await serveRouter(
    seed.factory(),
    seed.address,
    seed.port,
    shared: true,
  );

  final commands = ReceivePort();
  seed.ready.send(commands.sendPort);

  await for (final message in commands) {
    final request = message as List<Object?>;
    await server.close(drain: Duration(milliseconds: request[0]! as int));
    (request[1]! as SendPort).send(null);
    commands.close();
  }
}
