import 'dart:io';

import 'package:dust_server/server.dart';

/// Recording what was served.
///
/// `AccessLog` hands an [AccessRecord] to a callback rather than printing. That
/// is deliberate: a log line on stdout is one destination out of several, and a
/// service that wants JSON on stdout, a counter, and a sampled trace should not
/// have to fork the layer to get them.
///
/// Where it sits in the stack decides what it can see:
///
/// * **Above the routes** — as here — it records the 404s and 405s too. A
///   request that never reaches the log is one nobody can explain.
/// * Below a guard, it would record only the requests that got past it, which
///   hides exactly the traffic you want to look at during an incident.
///
/// > **Do not log the query string wholesale.** It carries API keys, session
/// > tokens, and reset links often enough that access logs are a recognised
/// > place credentials leak. This example logs the path and drops the query.
///
/// Run it with `dart run example/access_log.dart`:
///
/// ```bash
/// curl -s 'localhost:8080/notes?api_key=secret'
/// curl -s  localhost:8080/nothing
/// ```
///
/// The second line on stdout is the 404 — logged, with the key not written down:
///
/// ```json
/// {"method":"GET","path":"/notes","status":200}
/// {"method":"GET","path":"/nothing","status":404}
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp({void Function(AccessRecord)? onRecord}) {
  return Router()
    ..layer(const RequestId())
    ..layer(AccessLog(onRecord ?? printJson))
    ..route('/notes', get(listNotes));
}

/// `GET /notes`
List<String> listNotes(Request request) => const ['first'];

/// Writes one JSON object per request, which is what a log collector parses.
///
/// The path is recorded without its query. `record.path` is the raw path, so
/// anything after `?` never reaches the line.
void printJson(AccessRecord record) {
  stdout.writeln(
    '{"method":"${record.method}","path":"${record.path}",'
    '"status":${record.status},"ms":${record.duration.inMilliseconds}}',
  );
}
