import 'dart:io';

import 'package:dust_server/server.dart';

/// Letting a browser on another origin call this API.
///
/// CORS is not a server security feature. It is instructions **to a browser**
/// about what its JavaScript may read. A tightened CORS policy stops a page on
/// another site reading your responses; it stops nothing from `curl`, so it is
/// never a substitute for authentication.
///
/// Two kinds of request, and only the first is CORS-specific:
///
/// * A **preflight** — `OPTIONS` with `Access-Control-Request-Method` — is
///   answered by the layer and never reaches a handler.
/// * A **simple request** reaches the handler as normal and has the headers
///   added on the way out.
///
/// `Vary: Origin` matters as much as the allow header. Without it a cache can
/// serve one origin's response to another, which turns a correct policy into an
/// incorrect one at the CDN.
///
/// > **Credentials and `*` are mutually exclusive**, by browser rule, and `Cors`
/// > throws at construction rather than letting you find out from a browser
/// > console. Naming the origins is the fix, not a workaround.
///
/// Run it with `dart run example/cors.dart`:
///
/// ```bash
/// # a simple request from an allowed origin
/// curl -si localhost:8080/api/notes -H 'origin: https://app.example'
///
/// # from an origin that is not on the list
/// curl -si localhost:8080/api/notes -H 'origin: https://evil.example'
///
/// # a preflight, answered without reaching the handler
/// curl -si -X OPTIONS localhost:8080/api/notes \
///   -H 'origin: https://app.example' \
///   -H 'access-control-request-method: POST'
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
    ..layer(
      Cors(
        // Named, not `any()`. `any()` is right for a public read-only API and
        // wrong the moment a cookie or an Authorization header is involved.
        origins: const AllowedOrigins.only({
          'https://app.example',
          'http://localhost:3000',
        }),
        methods: const {'GET', 'POST', 'OPTIONS'},
        headers: const {'authorization', 'content-type'},
        // What JavaScript may read beyond the safelisted headers. Without this,
        // fetch() cannot see x-request-id even though it arrived.
        exposeHeaders: const {'x-request-id'},
        credentials: true,
        maxAge: const Duration(minutes: 10),
      ),
    )
    ..layer(const RequestId())
    ..route('/api/notes', get(listNotes).post(createNote, status: 201));
}

/// `GET /api/notes`
List<String> listNotes(Request request) => const ['first', 'second'];

/// `POST /api/notes`
Map<String, Object?> createNote(Request request) => const {'created': true};
