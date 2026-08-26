import 'dart:io';

import 'package:dust_server/server.dart';

/// Reading values out of the path.
///
/// Three forms, and the third is the one people miss:
///
/// * `{id}` matches one segment.
/// * `{id|\d+}` matches one segment **that matches the pattern**, so a route
///   can turn away `/orders/abc` before a handler runs.
/// * `{*rest}` matches everything left, slashes included, for proxying and for
///   serving files.
///
/// > **A parameter arrives percent-decoded, and that is not a formality.**
/// > `/files/a%2Fb` matches `{name}` as **one** segment — correctly — and hands
/// > the handler `a/b`, with a real slash in it. `%00` arrives as a NUL, `%20`
/// > as a space. None of that is a defect; it is what the value is. The defect
/// > would be joining it onto a filesystem path, an internal URL, or a command
/// > line. Store under an identifier you generated, or restrict the value at the
/// > route with `{name|[a-z0-9-]+}`, which refuses with 404 before a handler
/// > runs.
///
/// `request.path<int>('id')` coerces. A value that will not coerce is a **400**,
/// raised by the extractor, so a handler never sees a half-parsed request. Base
/// 10 only: `0x10` is not 16 here, because two URLs that mean the same number
/// would break caching.
///
/// Run it with `dart run example/path_params.dart`:
///
/// ```bash
/// curl -s localhost:8080/orders/41
/// curl -i localhost:8080/orders/abc        # 400, the coercion refused
/// curl -s localhost:8080/strict/41
/// curl -i localhost:8080/strict/abc        # 404, the route never matched
/// curl -s localhost:8080/files/css/app.css
/// curl -s localhost:8080/teams/dust/members/ada
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  return Router()
    ..route('/orders/{id}', get(readOrder))
    ..route(r'/strict/{id|\d+}', get(readStrict))
    ..route('/files/{*rest}', get(readFile))
    ..route('/teams/{team}/members/{member}', get(readMember));
}

/// `GET /orders/{id}` — coerced, so a bad value is a 400 before this runs.
Future<Map<String, Object?>> readOrder(Request request) async => {
      'id': await request.path<int>('id'),
    };

/// `GET /strict/{id|\d+}` — the route itself refuses a non-numeric id, with 404.
Future<Map<String, Object?>> readStrict(Request request) async => {
      'id': await request.path<int>('id'),
    };

/// `GET /files/{*rest}` — everything after the prefix, slashes kept.
Future<Map<String, Object?>> readFile(Request request) async => {
      'path': await request.path<String>('rest'),
    };

/// `GET /teams/{team}/members/{member}` — more than one, read by name.
Future<Map<String, Object?>> readMember(Request request) async => {
      'team': await request.path<String>('team'),
      'member': await request.path<String>('member'),
    };
