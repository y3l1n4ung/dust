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
/// pool, and nothing is shared between them; see [serveIsolates].
final class ServerIsolates {
  ServerIsolates._(this._isolates, this._handles, this.address, this.port);

  final List<Isolate> _isolates;
  final List<SendPort> _handles;
  final _dead = <int>{};
  final _workers = <_Worker>[];

  /// The address every isolate bound to.
  final InternetAddress address;

  /// The port every isolate shares.
  final int port;

  /// How many isolates were started.
  int get size => _isolates.length + 1;

  /// How many are still running.
  ///
  /// Lower than [size] once a spawned isolate has died. It cannot be replaced:
  /// killing an isolate does not release the socket it bound, so a new one
  /// cannot take the port back. Recovery is replacing the process, which is
  /// what a supervisor outside it does.
  ///
  /// Worth watching, because nothing else reports the loss — the port stays
  /// bound by the survivors and traffic keeps flowing at reduced capacity.
  int get alive => size - _dead.length;

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
    for (final worker in _workers) {
      worker.dispose();
    }
  }
}

/// Serves [factory] from [isolates] isolates, all sharing one port.
///
/// Dart runs one isolate on one thread, so a single server uses one core, and
/// isolates do not share memory. Using the rest of the machine means running
/// the server several times over rather than adding threads to it.
///
/// [serve] on its own uses one core. On a four-core machine that leaves
/// three idle under any load, and nothing reports it — the throughput ceiling
/// looks like the application's rather than the process's.
///
/// This is what `uvicorn --workers` and `gunicorn -w` do for Python, for the
/// same reason: the runtime cannot spread one server across cores, so it is run
/// several times behind a shared socket. A threaded runtime needs none of it,
/// so there is no equivalent to copy; this is here for the isolate model, not
/// as a convenience on top of [serve].
///
/// The operating system load-balances accepted connections across sockets bound
/// with `shared: true`, which is what lets several isolates answer one port.
///
/// ```dart
/// Router buildApp() => Router()..route('/', get(home));
///
/// void main() async {
///   final cluster = await serveIsolates(
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
Future<ServerIsolates> serveIsolates(
  RouterFactory factory,
  InternetAddress address,
  int port, {
  required int isolates,
  void Function(Object? error, Object? stackTrace)? onIsolateError,
}) async {
  if (isolates < 1) {
    throw ArgumentError.value(isolates, 'isolates', 'must be at least one');
  }

  final local = await serve(factory(), address, port, shared: true);

  final spawned = <Isolate>[];
  final handles = <SendPort>[];
  final workers = <_Worker>[];

  for (var i = 1; i < isolates; i++) {
    final worker = _Worker(i);
    final isolate = await Isolate.spawn(
      _serveInIsolate,
      _IsolateSeed(factory, address, local.port, worker.ready.sendPort),
      debugName: 'dust_server isolate $i',
      onExit: worker.exits.sendPort,
      onError: worker.exits.sendPort,
    );
    worker.isolate = isolate;

    final SendPort handle;
    try {
      handle = await worker.started.future;
    } on Object {
      // Nothing the caller knows about is serving yet, so leaving the local
      // server bound and the earlier isolates alive would leak a port and a
      // heap each.
      worker.dispose();
      isolate.kill(priority: Isolate.immediate);
      for (final earlier in workers) {
        earlier.dispose();
        earlier.isolate?.kill(priority: Isolate.immediate);
      }
      await local.close(drain: Duration.zero);
      rethrow;
    }

    handles.add(handle);
    worker.ready.close();
    spawned.add(isolate);
    workers.add(worker);
  }

  handles.add(_localHandle(local));
  final cluster = ServerIsolates._(spawned, handles, address, local.port)
    .._workers.addAll(workers);

  // An isolate that dies afterwards is otherwise invisible: the port stays
  // bound by the survivors, so traffic keeps flowing at reduced capacity with
  // nothing to say so. This cannot restart it — a replacement cannot rebind
  // the socket — but it can stop the loss being silent.
  for (var i = 0; i < workers.length; i++) {
    final index = i;
    workers[i].onDeath = (message) {
      cluster._dead.add(index);
      if (message is List && message.length == 2) {
        onIsolateError?.call(message.first, message.last);
      }
    };
  }

  return cluster;
}

/// One spawned isolate, and the two ports used to talk to it.
///
/// `exits` carries both startup failure and later death, because a
/// `ReceivePort` allows one subscription: the single listener below decides
/// which of the two it is by whether startup has finished.
final class _Worker {
  _Worker(this.index) {
    exits.listen((message) {
      if (!started.isCompleted) {
        started.completeError(
          StateError(
            'dust_server isolate $index died before it began serving: '
            '${message is List && message.isNotEmpty ? message.first : message}',
          ),
        );
        return;
      }
      onDeath?.call(message);
    });

    // `listen`, not `first`: closing the port after a failed start completes
    // `first` with a "No element" error nobody is waiting for any more.
    ready.listen((message) {
      if (!started.isCompleted) started.complete(message as SendPort);
    });
  }

  final int index;
  final ready = ReceivePort();
  final exits = ReceivePort();
  final started = Completer<SendPort>();

  Isolate? isolate;
  void Function(Object? message)? onDeath;

  void dispose() {
    ready.close();
    exits.close();
  }
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

final class _IsolateSeed {
  const _IsolateSeed(this.factory, this.address, this.port, this.ready);

  final RouterFactory factory;
  final InternetAddress address;
  final int port;
  final SendPort ready;
}

Future<void> _serveInIsolate(_IsolateSeed seed) async {
  final server = await serve(
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
