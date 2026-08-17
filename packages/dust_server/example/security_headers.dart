import 'dart:io';

import 'package:dust_server/server.dart';

/// The response headers a browser uses to lock a page down.
///
/// None of these make the server correct. Each turns a class of mistake into a
/// smaller one:
///
/// | Header | Without it |
/// | :--- | :--- |
/// | `X-Content-Type-Options: nosniff` | a text file a user uploaded can be sniffed as HTML and run |
/// | `X-Frame-Options: DENY` | your page can be framed, which is what clickjacking needs |
/// | `Referrer-Policy` | the full URL, tokens included, leaks to the next site |
/// | `Content-Security-Policy` | one injected `<script>` runs with your origin's privileges |
/// | `Strict-Transport-Security` | the first request each time can be plain HTTP |
///
/// The last two have **no defaults**, and that is deliberate:
///
/// * A wrong CSP breaks a working page, and a permissive one is theatre. It
///   belongs to the application, which knows what it loads.
/// * HSTS sent over plain HTTP does nothing, and sent from a host whose
///   certificate later lapses it locks users out of the site. Turn it on when
///   you mean it.
///
/// Passing `null` drops a header. A default that cannot be turned off is a
/// default that gets forked.
///
/// Run it with `dart run example/security_headers.dart`:
///
/// ```bash
/// curl -sI localhost:8080/page
/// curl -sI localhost:8080/api/notes    # no CSP: it is not an HTML document
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  // The API half gets the cheap headers and no CSP: a policy meant for a
  // document does nothing for a JSON response, and shipping one there trains
  // people to ignore the header.
  final api = Router()
    ..layer(const SecurityHeaders())
    ..route('/notes', get(listNotes));

  final pages = Router()
    ..layer(
      const SecurityHeaders(
        // Named sources, no 'unsafe-inline'. An allowlist that permits inline
        // scripts permits the injected one too.
        contentSecurityPolicy:
            "default-src 'self'; img-src 'self' data:; object-src 'none'; "
            "base-uri 'self'; frame-ancestors 'none'",
        // Commented rather than set: this example runs on plain HTTP, where the
        // header does nothing, and copying it into one that does is a decision.
        // strictTransportSecurity: 'max-age=63072000; includeSubDomains',
      ),
    )
    ..route('/page', get(page));

  return Router()
    ..nest('/api', api)
    ..merge(pages);
}

/// `GET /api/notes`
List<String> listNotes(Request request) => const ['first'];

/// `GET /page`
Response page(Request request) =>
    htmlResponse('<!doctype html><title>Locked down</title><h1>Hello</h1>');
