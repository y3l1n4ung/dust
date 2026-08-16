import 'package:shelf/shelf.dart';
import 'package:shelf_static/shelf_static.dart' as shelf_static;

/// Serves the files under [directory].
///
/// A thin pass to `shelf_static`, which already handles content types, ranges,
/// conditional requests, and refusing paths that climb out of the directory.
/// Mount it, so the handler sees paths relative to where it lives:
///
/// ```dart
/// final app = Router()
///   ..mount('/assets', staticFiles('web/assets'))
///   ..route('/', get(homePage));
/// ```
///
/// [defaultDocument] is what a directory request answers with, which is what
/// makes a single-page application work when the browser asks for `/`.
Handler staticFiles(
  String directory, {
  String? defaultDocument = 'index.html',
  bool listDirectories = false,
  bool serveFilesOutsidePath = false,
}) {
  return shelf_static.createStaticHandler(
    directory,
    defaultDocument: defaultDocument,
    listDirectories: listDirectories,
    serveFilesOutsidePath: serveFilesOutsidePath,
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
///   ..mount('/assets', staticFiles('web/assets'))
///   ..fallback(singlePageApp('web/index.html'));
/// ```
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
