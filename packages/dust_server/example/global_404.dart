import 'dart:io';

import 'package:dust_server/server.dart';

/// One fallback for everything unmatched.
///
/// Without a fallback, an unmatched path answers a plain JSON 404. `fallback`
/// takes it over — which matters most for a server doing two jobs at once, where
/// the right 404 depends on who is asking:
///
/// * a browser wants a page it can read;
/// * an API client wants JSON it can parse.
///
/// The fallback runs after matching, so it sees the paths nothing claimed and
/// nothing else. A 405 does not reach it: a path that exists for another method
/// is answered with `Allow`, which is more useful than a 404.
///
/// Run it with `dart run example/global_404.dart`:
///
/// ```bash
/// curl -s  localhost:8080/nothing                        # HTML
/// curl -s  localhost:8080/api/nothing                    # JSON
/// curl -si -X PUT localhost:8080/api/notes               # 405, not the fallback
/// curl -s  localhost:8080/nothing -H 'accept: application/json'
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  final api = Router()..route('/notes', get(listNotes));

  return Router()
    ..nest('/api', api)
    ..route('/', get(home))
    ..fallback(missing);
}

/// `GET /api/notes`
List<String> listNotes(Request request) => const ['first'];

/// `GET /`
Response home(Request request) => htmlResponse('<h1>Home</h1>');

/// Everything nothing else claimed.
///
/// The path decides the format, not the `Accept` header alone: a request under
/// `/api` is from a client whatever it says it accepts, and a browser following
/// a stale link sends `Accept: text/html` for a path that has nothing to do with
/// HTML. Checking both, path first, gets it right more often than either alone.
Response missing(Request request) {
  final path = '/${request.url.path}';
  final wantsJson = path.startsWith('/api/') ||
      (request.headers['accept'] ?? '').contains('application/json');

  if (wantsJson) {
    return const Rejection.notFound('no such route').intoResponse();
  }

  return htmlResponse(
    '<!doctype html><title>Not found</title>'
    '<h1>Not found</h1><p><a href="/">Home</a></p>',
    status: 404,
  );
}
