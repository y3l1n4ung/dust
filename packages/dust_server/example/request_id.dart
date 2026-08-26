import 'dart:io';

import 'package:dust_server/server.dart';

/// Giving every request an id that reaches the logs.
///
/// `RequestId` generates one when the client did not send it and **keeps the
/// one it did**. That is the whole feature: a gateway or a mobile client that
/// puts an id on the way in gets the same id back, so one identifier spans every
/// hop instead of each service inventing its own.
///
/// It goes on the response as well as into the context, because the id a user
/// reads off a screen is the one you need to search for.
///
/// For a full trace — parent spans, timing, sampling — see `tracing.dart`. This
/// is the small version, and for many services it is enough.
///
/// Run it with `dart run example/request_id.dart`:
///
/// ```bash
/// curl -si localhost:8080/notes
/// curl -si localhost:8080/notes -H 'x-request-id: from-the-gateway'
/// curl -s  localhost:8080/echo-id
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp({void Function(AccessRecord)? log}) {
  return Router()
    // Above the routes, so a 404 is logged too — a request that never reaches
    // the log is one nobody can explain.
    ..layer(const RequestId())
    ..layer(AccessLog(log ?? (record) => stdout.writeln(record)))
    ..route('/notes', get(listNotes))
    ..route('/echo-id', get(echoId));
}

/// `GET /notes`
List<String> listNotes(Request request) => const ['first'];

/// `GET /echo-id` — the same id the response header carries.
///
/// Reading it in the handler is what lets an error message quote the id the
/// caller can see, so a support conversation starts with a search rather than a
/// guess.
Map<String, Object?> echoId(Request request) => {
      'requestId': requestIdOf(request),
    };
