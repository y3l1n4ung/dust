import 'dart:io';

import 'package:dust_server/server.dart';

/// Compressing responses, and knowing when it declines to.
///
/// `Compression` gzips a response only when all of these hold. Each skip is a
/// decision, not an omission:
///
/// * the client sent `Accept-Encoding: gzip` — and `gzip;q=0` is a **refusal**,
///   not an absence, which a layer that only greps for the word gets wrong;
/// * the `content-type` is in `types` — a JPEG is already compressed, and
///   gzipping it spends CPU to add bytes;
/// * the length is at least `minimumBytes` — under a kilobyte the gzip header
///   costs more than it saves.
///
/// `Vary: Accept-Encoding` is appended, never replaced. A cache that misses it
/// will hand gzipped bytes to a client that cannot read them.
///
/// Run it with `dart run example/compression.dart`:
///
/// ```bash
/// # big enough, and the client accepts it
/// curl -sI localhost:8080/rows -H 'accept-encoding: gzip'
///
/// # a refusal, not an absence
/// curl -sI localhost:8080/rows -H 'accept-encoding: gzip;q=0'
///
/// # too small to be worth it
/// curl -sI localhost:8080/ping -H 'accept-encoding: gzip'
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  return Router()
    ..layer(const Compression())
    ..route('/rows', get(rows))
    ..route('/ping', get(ping));
}

/// `GET /rows` — comfortably over the threshold, and repetitive, so it shrinks.
List<Map<String, Object?>> rows(Request request) => [
      for (var index = 0; index < 60; index++)
        {
          'id': index,
          'title': 'a row with a reasonably long title',
          'done': false
        },
    ];

/// `GET /ping` — far under the threshold, so it is left alone.
Map<String, Object?> ping(Request request) => const {'ok': true};
