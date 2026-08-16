import 'dart:convert';

import 'package:shelf/shelf.dart';

/// Answers a request whose path exists under other methods.
///
/// A path that is served, just not this way, is a 405 carrying `Allow`. Only an
/// unknown path is a 404.
Response methodNotAllowed(String method, String path, List<String> allowed) {
  return Response(
    405,
    body: jsonEncode({'error': '$method is not allowed on $path'}),
    headers: {
      'content-type': 'application/json',
      'allow': allowed.join(', '),
    },
  );
}

/// Answers a request for a path nothing serves.
Response notFound(String path) {
  return Response.notFound(
    jsonEncode({'error': 'no route for $path'}),
    headers: const {'content-type': 'application/json'},
  );
}
