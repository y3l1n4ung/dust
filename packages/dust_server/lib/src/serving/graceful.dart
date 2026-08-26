import 'dart:async';
import 'dart:io';

import 'package:shelf/shelf.dart';
import 'package:shelf/shelf_io.dart' as shelf_io;

import '../router/middleware.dart';
import '../router/router_base.dart';
import 'background.dart';

/// A running server that can be stopped without dropping work.
///
/// `HttpServer.close` stops accepting and, unforced, waits for open
/// connections. What it does not do is tell you how many requests are still in
/// flight, which is what a deployment needs before it takes the process away.
final class ServerHandle {
  ServerHandle._(this._server, this._inFlight, this._background, this._layers)
      : address = _server.address,
        port = _server.port;

  final HttpServer _server;
  final _InFlight _inFlight;
  final BackgroundTasks? _background;
  final List<DisposableLayer> _layers;

  /// The address the server bound to.
  ///
  /// Captured when the server started, so it stays readable after [close];
  /// asking `HttpServer` directly throws once the socket is gone.
  final InternetAddress address;

  /// The port the server bound to.
  final int port;

  /// How many requests are being handled right now.
  int get inFlight => _inFlight.count;

  /// How many background tasks are still running, when a registry was passed.
  int get pendingTasks => _background?.pending ?? 0;

  /// Stops accepting, then waits for the requests already accepted.
  ///
  /// When a [BackgroundTasks] was passed to [serve], its work is drained
  /// too, and within the same [drain] budget rather than a second one — the
  /// platform kills the process on its own schedule, and two budgets in series
  /// exceed it.
  ///
  /// Returns `true` when everything finished within [drain], and `false` when
  /// the deadline passed with work still running, at which point the caller
  /// decides whether to wait longer or exit anyway.
  Future<bool> close({Duration drain = const Duration(seconds: 30)}) async {
    await _server.close();

    final deadline = Stopwatch()..start();
    final requestsSettled = await _inFlight.settled(drain);

    final background = _background;
    var tasksSettled = true;

    if (background != null) {
      // Whatever is left of the budget. A request that used all of it leaves
      // nothing, and `close` reports the failure rather than waiting twice as
      // long as it was told to.
      final remaining = drain - deadline.elapsed;
      tasksSettled = await background.close(
        within: remaining.isNegative ? Duration.zero : remaining,
      );
    }

    // After the tasks, because a background task may still be using whatever a
    // layer owns. Always, because a server with no background work still has
    // layers to release — an early return here left them open.
    await _disposeLayers();

    return requestsSettled && tasksSettled;
  }

  /// Releases every [DisposableLayer] on the router, once.
  ///
  /// A layer that throws does not stop the others: shutdown has already
  /// stopped accepting, and losing the rest of the teardown to one bad
  /// `dispose` is worse than the error itself.
  Future<void> _disposeLayers() async {
    for (final layer in _layers) {
      try {
        await layer.dispose();
      } catch (_) {
        // Reported by the layer if it wants to be; shutdown continues.
      }
    }
    _layers.clear();
  }
}

/// Serves [router], counting requests so shutdown can wait for them.
///
/// [shared] lets several isolates bind the same port, which is what
/// `serveIsolates` uses; on its own a single server has no reason to set it.
///
/// [background] is drained alongside the requests. When it is omitted, a
/// [BackgroundTasks] attached to [router] with `withState` is used instead — so
/// a clustered server, whose registry is built inside each isolate, needs no
/// extra wiring to be drained.
///
/// ```dart
/// final server = await serve(app, InternetAddress.anyIPv4, 8080);
/// await ProcessSignal.sigterm.watch().first;
/// await server.close(drain: const Duration(seconds: 15));
/// ```
Future<ServerHandle> serve(
  Router router,
  InternetAddress address,
  int port, {
  SecurityContext? securityContext,
  bool shared = false,
  BackgroundTasks? background,
}) async {
  final inFlight = _InFlight();
  // Falling back to the router's own state means `withState(tasks)` is enough,
  // and a clustered server — where the registry is built inside each isolate and
  // cannot be handed in from outside — drains its tasks without any extra
  // wiring.
  final tasks = background ?? backgroundTasksIn(router);
  final handler = router.handler;

  final server = await shelf_io.serve(
    (request) async {
      inFlight.enter();
      try {
        return _flushIfStreamed(await handler(request));
      } finally {
        inFlight.leave();
      }
    },
    address,
    port,
    securityContext: securityContext,
    shared: shared,
  );

  return ServerHandle._(server, inFlight, tasks, disposableLayersIn(router));
}

/// Turns off output buffering for a response whose length is not known.
///
/// `shelf` buffers a streamed body and flushes when the stream ends, which for a
/// download makes the client wait for all of it and for an endless stream means
/// never. `streamed` and `eventStream` opt out themselves; this covers a
/// `Response` built by hand, including one from a mounted third-party handler
/// that has never heard of Dust.
///
/// Only when the length is unknown. A response with a `content-length` is a
/// single write, where buffering costs nothing and helps.
///
/// A response that set the key itself is left alone, so `bufferOutput: true` on
/// a chunked response remains sayable — worth having for a body that arrives as
/// very many tiny chunks, where one write each is slower than one write.
Response _flushIfStreamed(Response response) {
  if (response.contentLength != null) return response;
  if (response.context.containsKey(_bufferOutputKey)) return response;

  return response.change(context: {_bufferOutputKey: false});
}

/// The context key `shelf_io` reads to decide whether to buffer.
const _bufferOutputKey = 'shelf.io.buffer_output';

final class _InFlight {
  int count = 0;
  Completer<void>? _idle;

  void enter() => count++;

  void leave() {
    count--;
    if (count == 0) {
      _idle?.complete();
      _idle = null;
    }
  }

  Future<bool> settled(Duration within) async {
    if (count == 0) return true;

    final idle = _idle ??= Completer<void>();
    try {
      await idle.future.timeout(within);
      return true;
    } on TimeoutException {
      return false;
    }
  }
}
