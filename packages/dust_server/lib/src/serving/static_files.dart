import 'package:shelf/shelf.dart';
import 'package:shelf_static/shelf_static.dart' as shelf_static;

/// Files a build regenerates without putting a content hash in their name.
///
/// A cache that keeps one of these keeps the whole application: the browser
/// holds a stale document, that document asks for the entry script it
/// remembers, and the new build is never fetched. It shows up a day after a
/// deploy as "users are on the old version and a hard refresh fixes it".
///
/// These names cover what the common toolchains emit. Bundlers that hash every
/// asset — Vite, Rollup, esbuild — need only the document and the service
/// worker; Flutter also ships unhashed entry files. Pass `revalidate:` to
/// replace the set when a toolchain uses other names.
const defaultRevalidatedFiles = <String>{
  'index.html',
  'manifest.json',
  'version.json',
  'service-worker.js',
  'sw.js',
  'flutter.js',
  'flutter_bootstrap.js',
  'flutter_service_worker.js',
  'main.dart.js',
  'main.dart.mjs',
  'main.dart.wasm',
};

/// Serves the files under [directory].
///
/// Without [html] this is a thin pass to `shelf_static`, which already handles
/// content types, ranges, conditional requests, and refusing paths that climb
/// out of the directory. Mount it, so the handler sees paths relative to where
/// it lives:
///
/// ```dart
/// final app = Router()
///   ..mount('/assets', staticFiles('web/assets'))
///   ..route('/', get(homePage));
/// ```
///
/// ## HTML mode
///
/// [html] turns a directory of files into a served application, the way
/// Starlette's `StaticFiles(html=True)` does. Use it for build output —
/// `build/web`, `dist`, `build` — from any toolchain that ships a document
/// plus assets and routes in the browser: Flutter web, React, Vue, Svelte:
///
/// ```dart
/// final app = Router()
///   ..nest('/api', apiRoutes)
///   ..fallback(staticFiles('build/web', html: true));
/// ```
///
/// Three things it adds:
///
/// 1. **Client-side routes reach the application.** `/orders/42` exists only
///    in the browser's router, so serving files alone answers 404 and a shared
///    link is broken. HTML mode answers [defaultDocument] instead, and the
///    address bar keeps the path that was asked for.
/// 2. **The shell is revalidated and the rest is not.** Anything named in
///    [revalidate] answers `no-cache`, which still allows a 304; everything
///    else answers `immutable` for [immutableFor], so a repeat visit costs
///    nothing.
/// 3. **Only `GET` and `HEAD` are answered.** A `POST` to a path no API route
///    matched is a wrong request, and handing it an HTML document would hide
///    that behind a 200.
///
/// Set [crossOriginIsolated] for an application that needs
/// `SharedArrayBuffer`, which a Flutter `--wasm` build does. It is off by
/// default because the headers also block third-party images, fonts, and
/// iframes that do not opt in with CORP.
Handler staticFiles(
  String directory, {
  bool html = false,
  String? defaultDocument = 'index.html',
  Set<String> revalidate = defaultRevalidatedFiles,
  Duration immutableFor = const Duration(days: 365),
  bool crossOriginIsolated = false,
  bool listDirectories = false,
  bool serveFilesOutsidePath = false,
}) {
  final files = shelf_static.createStaticHandler(
    directory,
    defaultDocument: defaultDocument,
    listDirectories: listDirectories,
    serveFilesOutsidePath: serveFilesOutsidePath,
  );
  if (!html) return files;

  final document = defaultDocument ?? 'index.html';
  final immutable = 'public, max-age=${immutableFor.inSeconds}, immutable';

  Map<String, String> headersFor(String path) {
    // A directory request resolved through the default document, so `/` and
    // `/admin/` are the document however the URL was spelled. Reading the last
    // segment alone would leave the root cached for a year, which is the
    // failure HTML mode exists to prevent.
    final name =
        path.isEmpty || path.endsWith('/') ? document : path.split('/').last;
    return {
      'cache-control': revalidate.contains(name) ? 'no-cache' : immutable,
      if (crossOriginIsolated) ...const {
        'cross-origin-opener-policy': 'same-origin',
        'cross-origin-embedder-policy': 'require-corp',
      },
    };
  }

  return (Request request) async {
    if (request.method != 'GET' && request.method != 'HEAD') {
      return Response.notFound('no route for /${request.url.path}');
    }

    final direct = await files(request);
    if (direct.statusCode != 404) {
      return direct.change(headers: headersFor(request.url.path));
    }

    // Nothing on disk matched, so the path belongs to the browser's router.
    final shell = await files(_asDocumentRequest(request, document));
    if (shell.statusCode == 404) return shell;

    return shell.change(headers: headersFor(document));
  };
}

/// Rewrites [request] to ask for [document] instead of what it asked for.
///
/// The mount prefix is preserved, so a build served under `/app` still
/// resolves its own document rather than escaping to the root. The browser
/// keeps the path it asked for; only what the file handler resolves changes.
Request _asDocumentRequest(Request request, String document) {
  return Request(
    request.method,
    request.requestedUri.replace(
      path: '${request.handlerPath}$document',
      query: '',
    ),
    headers: request.headers,
    context: request.context,
    handlerPath: request.handlerPath,
  );
}

/// Serves one file for every document request, for a single-page application.
///
/// A client-side router owns the path, so the server answers the same document
/// whatever the browser asks for. Register it as a fallback, after the routes
/// that should still win:
///
/// ```dart
/// final app = Router()
///   ..mount('/api', apiRoutes)
///   ..fallback(singlePageApp('web/index.html'));
/// ```
///
/// This answers one file and nothing else. For a whole build directory, which
/// is the usual case, use `staticFiles(directory, html: true)` instead — it
/// serves the assets beside the document and gets the cache policy right.
///
/// Only `GET` and `HEAD` are answered. A `POST` to a path nothing matched is a
/// wrong request, not a page view, and handing it an HTML document would hide
/// that.
Handler singlePageApp(String file) {
  final url = file.split('/').last;
  final inner = shelf_static.createFileHandler(file, url: url);

  return (Request request) {
    if (request.method != 'GET' && request.method != 'HEAD') {
      return Response.notFound('no route for /${request.url.path}');
    }

    // `createFileHandler` only answers at the file's own URL, so the request is
    // pointed at it; the client-side route stays in the browser's address bar.
    return inner(
      Request(
        request.method,
        request.requestedUri.replace(path: '/$url'),
        headers: request.headers,
        context: request.context,
      ),
    );
  };
}
