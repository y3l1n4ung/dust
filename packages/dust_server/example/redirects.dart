import 'dart:io';

import 'package:dust_server/server.dart';

/// Redirecting, and which status to use.
///
/// The status is not a formality — it decides whether the method survives and
/// whether anything caches the answer:
///
/// | Helper | Status | Method | Cached |
/// | :--- | :-- | :--- | :--- |
/// | `Redirect.to` | 303 | forced to `GET` | no |
/// | `Redirect.temporary` | 307 | kept | no |
/// | `Redirect.permanent` | 308 | kept | yes |
/// | `Redirect.found` | 302 | kept in theory, `GET` in practice | no |
/// | `Redirect.movedPermanently` | 301 | same, and cached forever | yes |
///
/// **303 after a successful POST.** It turns the follow-up into a `GET`, so the
/// browser's back button and reload do not re-submit the form. That pattern has
/// a name — POST/redirect/GET — and skipping it is why double-charged orders
/// exist.
///
/// **301 and 308 are close to permanent.** Browsers cache them, often past a
/// restart, so a wrong one is a URL you cannot take back for the lifetime of
/// every client that saw it. Send 302 or 307 while you are unsure.
///
/// The `Location` value is stripped of `\r`, `\n`, and NUL. That header is one of
/// the few places a caller-supplied string reaches a header, and a newline there
/// would let a client inject headers of its own.
///
/// Run it with `dart run example/redirects.dart`:
///
/// ```bash
/// curl -si -X POST localhost:8080/notes -d 'title=x' # 303 to /notes/1
/// curl -si localhost:8080/old-home                   # 308
/// curl -si localhost:8080/maintenance                # 307
/// curl -si 'localhost:8080/search?q=a%0d%0aX-Evil:1' # the newline is stripped
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
    ..route('/notes', post(createNote))
    ..route('/notes/{id}', get(readNote))
    ..route('/old-home', get(movedHome))
    ..route('/maintenance', get(maintenance))
    ..route('/search', get(search));
}

/// `POST /notes` — POST/redirect/GET, so a reload does not re-submit.
Redirect createNote(Request request) => Redirect.to('/notes/1');

/// `GET /notes/{id}`
Future<Map<String, Object?>> readNote(Request request) async => {
      'id': await request.path<int>('id'),
    };

/// `GET /old-home` — the URL is gone for good, and callers may cache that.
Redirect movedHome(Request request) => Redirect.permanent('/');

/// `GET /maintenance` — today only, so nothing should remember it.
Redirect maintenance(Request request) => Redirect.temporary('/');

/// `GET /search?q=` — the query is echoed into `Location`, which is why the
/// helper strips control characters rather than trusting the caller.
Future<Redirect> search(Request request) async {
  final term = await request.query<String>('q');

  return Redirect.to('/notes?q=${Uri.encodeQueryComponent(term)}');
}
