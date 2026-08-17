import 'dart:io';

import 'package:dust_server/server.dart';

/// Reading the query string.
///
/// Four things it can be asked for, and the type says which:
///
/// * `query<int>('page')` is **required**. Absent is a 400.
/// * `query<int?>('page')` is optional — a nullable type is how you say so, and
///   there is no separate call for it.
/// * `queryList<String>('tag')` takes **every** value of a repeated key.
///   `query<String>` on `?tag=a&tag=b` gives you one of them and silently drops
///   the other, which is the bug this exists to prevent.
/// * `rawQuery()` is the undecoded string, for signing and for proxying, where
///   re-encoding would change the bytes you were meant to pass on.
///
/// `queryList` has no shortcut on the request, so it goes through
/// `request.extract(...)` — the extractor classes are the whole vocabulary, and
/// the shortcuts cover only the common ones.
///
/// Run it with `dart run example/query_params.dart`:
///
/// ```bash
/// curl -s 'localhost:8080/search?q=shirt&page=2'
/// curl -s 'localhost:8080/search?q=shirt'          # page defaults
/// curl -i 'localhost:8080/search'                  # 400, q is required
/// curl -s 'localhost:8080/filter?tag=red&tag=blue'
/// curl -s 'localhost:8080/raw?a=1&b=%20two'
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
    ..route('/search', get(search))
    ..route('/filter', get(filter))
    ..route('/raw', get(raw));
}

/// `GET /search?q=&page=` — one required, one optional with a default.
Future<Map<String, Object?>> search(Request request) async => {
      'q': await request.query<String>('q'),
      'page': await request.query<int?>('page') ?? 1,
    };

/// `GET /filter?tag=&tag=` — every value, not the last one.
Future<Map<String, Object?>> filter(Request request) async => {
      'tags': await request.extract(queryList<String>('tag')),
    };

/// `GET /raw` — the query exactly as it arrived.
Future<Map<String, Object?>> raw(Request request) async => {
      'query': await request.rawQuery(),
    };
