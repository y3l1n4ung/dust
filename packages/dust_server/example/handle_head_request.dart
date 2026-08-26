import 'dart:io';

import 'package:dust_server/server.dart';

/// Answering `HEAD` from a `GET` handler.
///
/// The router answers `HEAD` from the matching `GET` route and drops the body,
/// so `HEAD` costs you nothing to support and appears in `Allow` on a 405. A
/// client uses it to check a length, a type, or an `ETag` without paying for the
/// payload, and monitoring tools reach for it constantly.
///
/// **The handler still runs.** Only the body is discarded, which has one
/// consequence worth planning for: a `GET` that increments a counter or writes an
/// audit row does so for every `HEAD` too. If that is not wanted, the work does
/// not belong in a `GET`.
///
/// Declaring an explicit `HEAD` route overrides the automatic one. Worth it only
/// when the cheap answer is genuinely cheaper — a length you already know, rather
/// than a query you would have run.
///
/// Run it with `dart run example/handle_head_request.dart`:
///
/// ```bash
/// curl -s  localhost:8080/notes
/// curl -sI localhost:8080/notes            # same headers, no body
/// curl -sI localhost:8080/report           # the explicit HEAD route
/// curl -si -X PUT localhost:8080/notes     # Allow lists HEAD
/// curl -s  localhost:8080/counted          # the counter
/// curl -sI localhost:8080/counted          # which HEAD also advances
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// How many times the counted route ran, `HEAD` included.
final counter = Counter();

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  return Router()
    ..route('/notes', get(listNotes))
    ..route('/report', get(report).head(reportSize))
    ..route('/counted', get(counted))
    ..withState(counter);
}

/// `GET /notes`
List<String> listNotes(Request request) => const ['first', 'second'];

/// `GET /report` — expensive enough that a client may want to ask first.
Response report(Request request) =>
    textResponse('a report with a known length\n');

/// `HEAD /report` — the length without building the report.
Response reportSize(Request request) => Response.ok(
      null,
      headers: const {
        'content-type': 'text/plain; charset=utf-8',
        'content-length': '28',
      },
    );

/// `GET /counted` — and a reminder that `HEAD` runs the handler too.
Future<Map<String, Object?>> counted(Request request) async {
  final counter = await request.state<Counter>();

  return {'calls': ++counter.calls};
}

/// A counter, standing in for anything a `GET` should not have been doing.
final class Counter {
  /// How many times it has run.
  int calls = 0;
}
