import 'dart:async';
import 'dart:io';

import 'package:dust_server/server.dart';

/// Cutting off a request that takes too long.
///
/// Without a deadline, a handler that never completes holds its connection
/// forever, and enough of them exhaust the server. The body limit bounds how
/// much a request may send; this bounds how long it may take.
///
/// > **The handler keeps running.** Dart cannot cancel a `Future`, so the 503
/// > goes back while the work continues in the background. That has two
/// > consequences worth planning for: the work still costs what it was going to
/// > cost, and a write it was part-way through **still lands** after the client
/// > was told the request failed. Put the deadline above work you can afford to
/// > abandon, and make the writes underneath it idempotent.
///
/// > **It bounds producing the response, not sending it.** A handler that
/// > returns immediately with a streamed body has already met the deadline, and
/// > the stream then runs unbounded — so this layer over an SSE endpoint
/// > protects nothing. Bound the stream itself.
///
/// `onTimeout` is the hook for recording that it happened — a counter, a log
/// line — because a 503 nobody counted is an outage nobody noticed.
///
/// Run it with `dart run example/request_timeout.dart`:
///
/// ```bash
/// curl -s  localhost:8080/quick
/// curl -si localhost:8080/slow    # 503 after ~200ms
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp({void Function(Request)? onTimeout}) {
  return Router()
    ..layer(
      RequestTimeout(
        const Duration(milliseconds: 200),
        onTimeout: onTimeout ??
            (request) => stdout.writeln('timed out: ${request.url.path}'),
      ),
    )
    ..route('/quick', get(quick))
    ..route('/slow', get(slow));
}

/// `GET /quick`
Map<String, Object?> quick(Request request) => const {'ok': true};

/// `GET /slow` — over the budget, so the client gets a 503.
Future<Map<String, Object?>> slow(Request request) async {
  await Future<void>.delayed(const Duration(seconds: 2));

  return const {'ok': true};
}
