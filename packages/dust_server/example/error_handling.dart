import 'dart:io';

import 'package:dust_server/server.dart';

/// What happens when a handler throws.
///
/// Anything that escapes a handler is caught and answered as **500**, and the
/// body is always the same: `{"error":"Internal server error"}`. The detail goes
/// to `onError` and nowhere else.
///
/// > **That is the security property.** An exception message routinely carries a
/// > file path, a SQL fragment, a connection string, or a stack trace naming
/// > every package you depend on. Returning it is free reconnaissance, and it is
/// > the default in more frameworks than it should be.
///
/// Three ways a failure reaches the client, and only the first is a fault:
///
/// * **A throw** — a bug, or something downstream broke. 500, detail withheld.
/// * **A thrown `Rejection`** — a decision, not a fault. Keeps its own status.
/// * **`Err(rejection)`** — the same, said in the return type, which is better
///   because the signature tells a reader it can fail.
///
/// `onError` is where a failure becomes visible: a logger, Sentry, a counter. A
/// 500 nobody was told about is an outage nobody noticed.
///
/// Run it with `dart run example/error_handling.dart`:
///
/// ```bash
/// curl -s  localhost:8080/ok
/// curl -si localhost:8080/throws     # 500, and nothing about what broke
/// curl -si localhost:8080/rejects    # 409, because it said so
/// curl -si localhost:8080/returns    # 409, said in the return type
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp({void Function(Object error, StackTrace stack)? onError}) {
  return Router(
    // Scoped to this router rather than set globally, so two applications in
    // one isolate do not overwrite each other's sink.
    onError: onError ?? report,
  )
    ..route('/ok', get(ok))
    ..route('/throws', get(throws))
    ..route('/rejects', get(rejects))
    ..route('/returns', get(returns));
}

/// `GET /ok`
Map<String, Object?> ok(Request request) => const {'ok': true};

/// `GET /throws` — a bug. The client learns nothing; `onError` learns everything.
Future<Map<String, Object?>> throws(Request request) async {
  throw StateError('the connection string is postgres://user:hunter2@db');
}

/// `GET /rejects` — a decision, thrown.
Future<Map<String, Object?>> rejects(Request request) async {
  throw const Rejection.conflict('that name is taken');
}

/// `GET /returns` — the same decision, in the return type.
///
/// Better than throwing it: the signature says the call can fail, so a reader
/// does not have to guess and a caller cannot forget.
Future<Result<Map<String, Object?>, Rejection>> returns(Request request) async {
  return const Err(Rejection.conflict('that name is taken'));
}

/// Where a fault becomes visible.
void report(Object error, StackTrace stack) {
  // A real application sends this to a logger or an error tracker. Printing it
  // is what makes the example legible.
  stderr.writeln('unhandled: $error');
}
