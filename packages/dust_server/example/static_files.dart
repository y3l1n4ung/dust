import 'dart:io';

import 'package:dust_server/server.dart';

/// Serving a built front end, single-page routes included.
///
/// `html: true` is the mode a Flutter, React, or Vue build needs. Without it a
/// deep link like `/orders/41` is a 404: there is no such file, and the router
/// the browser would have used has not loaded yet. With it, the default document
/// answers and the client-side router takes over.
///
/// The naming follows Starlette's `html=True` rather than inventing a term, and
/// it is not Flutter-specific — any single-page build works the same way.
///
/// The cache policy is the part that decides whether a deploy is visible:
///
/// * A **fingerprinted** asset — `main.a1b2c3.js` — is immutable. Its name
///   changes when its content does, so it can be cached for a year.
/// * The **default document** and the service worker must be revalidated every
///   time. Cache `index.html` for a year and users keep the old app until the
///   cache expires, pointing at asset names that no longer exist.
///
/// > **`serveFilesOutsidePath` stays off.** On it, a crafted path can read files
/// > above the directory. There is no legitimate use for it in a served build.
///
/// `crossOriginIsolated` sends COOP and COEP, which a Flutter web build needs for
/// `SharedArrayBuffer`. It also blocks cross-origin resources that do not opt in,
/// so turn it on when you need it and not by default.
///
/// Run it with `dart run example/static_files.dart`:
///
/// ```bash
/// curl -si localhost:8080/                  # the document, revalidated
/// curl -si localhost:8080/orders/41         # the same document, deep link
/// curl -sI localhost:8080/main.a1b2c3.js    # immutable for a year
/// curl -sI localhost:8080/../pubspec.yaml   # not served
/// ```
Future<void> main() async {
  final root = await _buildDirectory();
  final server =
      await serve(buildApp(root.path), InternetAddress.anyIPv4, 8080);
  stdout.writeln('serving ${root.path} on http://localhost:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
///
/// The API is declared **first**, and that is load-bearing. A `mount('/')`
/// claims every path, and the first declaration wins — so a static handler
/// written above the API would answer `/api/notes` with the document.
///
/// One consequence to plan for: `/api/nothing` reaches the mount too, and gets
/// the document rather than a JSON 404. Add a catch-all under `/api` when a
/// client should be told, as `global_404.dart` does.
Router buildApp(String directory) {
  final api = Router()..route('/notes', get(listNotes));

  return Router()
    ..nest('/api', api)
    ..mount(
      '/',
      staticFiles(
        directory,
        html: true,
        // Never cached hard: the document is how a browser learns the new asset
        // names, so a stale one pins users to a deploy you have replaced.
        revalidate: const {'index.html', 'flutter_service_worker.js'},
        immutableFor: const Duration(days: 365),
      ),
    );
}

/// `GET /api/notes`
List<String> listNotes(Request request) => const ['first'];

/// Writes a throwaway build so the example runs with nothing checked in.
Future<Directory> _buildDirectory() async {
  final root = await Directory.systemTemp.createTemp('dust-static-');
  await File('${root.path}/index.html').writeAsString(
    '<!doctype html><title>App</title><div id="app">loading</div>',
  );
  await File('${root.path}/main.a1b2c3.js')
      .writeAsString('console.log("fingerprinted");');
  return root;
}
