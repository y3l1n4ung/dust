import 'dart:io';

import 'package:dust_server/server.dart';

/// Turning a missing value into `None` instead of a 400.
///
/// Three ways to say "this might not be there", and they mean different things:
///
/// | Written | Absent becomes | Malformed becomes |
/// | :--- | :--- | :--- |
/// | `query<int>('page')` | 400 | 400 |
/// | `query<int?>('page')` | `null` | 400 |
/// | `optional(query<int>('page'))` | `None` | `None` |
///
/// The third row is the one to be careful with. `optional` swallows **any**
/// client-error rejection, so `?page=abc` comes back as `None` rather than as a
/// complaint — a client with a bug gets silence and page one. Use a nullable
/// type when absent is fine but wrong is not, which is most of the time. Reach
/// for `optional` when you are composing, and a failure genuinely means "try
/// something else".
///
/// Run it with `dart run example/optional_extraction.dart`:
///
/// ```bash
/// curl -s 'localhost:8080/strict?page=2'
/// curl -i 'localhost:8080/strict'          # 400
/// curl -s 'localhost:8080/nullable'        # null, then defaulted
/// curl -i 'localhost:8080/nullable?page=x' # 400, malformed is still an error
/// curl -s 'localhost:8080/optional?page=x' # None — the bug is swallowed
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
    ..route('/strict', get(strict))
    ..route('/nullable', get(nullable))
    ..route('/optional', get(optionally));
}

/// `GET /strict?page=` — required.
Future<Map<String, Object?>> strict(Request request) async => {
      'page': await request.query<int>('page'),
    };

/// `GET /nullable?page=` — absent is fine, malformed is not.
Future<Map<String, Object?>> nullable(Request request) async => {
      'page': await request.query<int?>('page') ?? 1,
    };

/// `GET /optional?page=` — absent and malformed are both `None`.
Future<Map<String, Object?>> optionally(Request request) async {
  final page = await request.extract(optional(query<int>('page')));

  return {
    'page': switch (page) {
      Some(:final value) => value,
      None() => 'none',
    },
  };
}
