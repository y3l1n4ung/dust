import 'dart:async';
import 'dart:io';

import 'package:shelf/shelf_io.dart' as shelf_io;

import '../router/router_base.dart';

/// A running server that can be stopped without dropping work.
///
/// `HttpServer.close` stops accepting and, unforced, waits for open
/// connections. What it does not do is tell you how many requests are still in
/// flight, which is what a deployment needs before it takes the process away.
final class ServerHandle {
  ServerHandle._(this._server, this._inFlight)
      : address = _server.address,
        port = _server.port;

  final HttpServer _server;
  final _InFlight _inFlight;

  /// The address the server bound to.
  ///
  /// Captured when the server started, so it stays readable after [close];
  /// asking `HttpServer` directly throws once the socket is gone.
  final InternetAddress address;

  /// The port the server bound to.
  final int port;

  /// How many requests are being handled right now.
  int get inFlight => _inFlight.count;

  /// Stops accepting, then waits for the requests already accepted.
  ///
  /// Returns `true` when everything finished within [drain], and `false` when
  /// the deadline passed with work still running, at which point the caller
  /// decides whether to wait longer or exit anyway.
  Future<bool> close({Duration drain = const Duration(seconds: 30)}) async {
    await _server.close();
    return _inFlight.settled(drain);
  }
}

/// Serves [router], counting requests so shutdown can wait for them.
///
/// [shared] lets several isolates bind the same port, which is what
/// `serveCluster` uses; on its own a single server has no reason to set it.
///
/// ```dart
/// final server = await serveRouter(app, InternetAddress.anyIPv4, 8080);
/// await ProcessSignal.sigterm.watch().first;
/// await server.close(drain: const Duration(seconds: 15));
/// ```
Future<ServerHandle> serveRouter(
  Router router,
  Object address,
  int port, {
  SecurityContext? securityContext,
  bool shared = false,
}) async {
  final inFlight = _InFlight();
  final handler = router.handler;

  final server = await shelf_io.serve(
    (request) async {
      inFlight.enter();
      try {
        return await handler(request);
      } finally {
        inFlight.leave();
      }
    },
    address,
    port,
    securityContext: securityContext,
    shared: shared,
  );

  return ServerHandle._(server, inFlight);
}

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
