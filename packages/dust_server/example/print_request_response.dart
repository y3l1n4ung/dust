import 'dart:io';

import 'package:dust_server/server.dart';

/// Seeing what actually went over the wire.
///
/// When a client swears it sent the right thing, this is the layer that settles
/// it. The trick is that a body can be read **once** — so a layer that prints it
/// has consumed it, and the handler below gets nothing.
///
/// The fix is to read it and hand a fresh request down carrying the same bytes,
/// which is what `request.change(body:)` is for. The response side is the same
/// problem in reverse.
///
/// > **Never leave this on.** Bodies carry passwords, tokens, card numbers, and
/// > personal data, and a log is retained for months and read by more people
/// > than your database is. This example redacts the obvious headers and still
/// > should not run in production — the safe version logs metadata only.
///
/// Buffering also removes streaming: a large upload or an SSE response is held
/// in memory while this is in the stack, which is another reason it is a
/// development tool.
///
/// Run it with `dart run example/print_request_response.dart`:
///
/// ```bash
/// curl -s -X POST localhost:8080/notes -H 'content-type: application/json' \
///   -H 'authorization: Bearer secret-token' -d '{"title":"buy milk"}'
/// ```
///
/// ```text
/// --> POST /notes {content-type: application/json, authorization: <redacted>}
/// --> {"title":"buy milk"}
/// <-- 201 {"id":1,"title":"buy milk"}
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp({void Function(String line)? log}) {
  return Router()
    ..layer(PrintRequestResponse(log ?? stdout.writeln))
    ..route('/notes', post(createNote, status: 201));
}

/// `POST /notes`
Future<Map<String, Object?>> createNote(Request request) async {
  final title = await request.body((json) => json['title']! as String);

  return {'id': 1, 'title': title};
}

/// Prints each request and response, bodies included.
final class PrintRequestResponse implements Layer {
  /// Writes each line with [write].
  const PrintRequestResponse(this.write, {this.limit = 4096});

  /// Where the lines go.
  final void Function(String line) write;

  /// How much of a body to print before truncating.
  ///
  /// Without it, one large upload floods the log and takes the disk with it.
  final int limit;

  /// Headers never printed, whatever they contain.
  static const redacted = {
    'authorization',
    'cookie',
    'set-cookie',
    'x-api-key'
  };

  @override
  Middleware toMiddleware() {
    return (Handler inner) {
      return (Request request) async {
        final body = await request.readAsString();

        write('--> ${request.method} /${request.url.path} '
            '${_safeHeaders(request.headers)}');
        if (body.isNotEmpty) write('--> ${_clip(body)}');

        // A body reads once. The handler below gets a request carrying the same
        // bytes; without this it would find an empty body and a puzzling 400.
        final response = await inner(request.change(body: body));

        final answered = await response.readAsString();
        write('<-- ${response.statusCode} ${_clip(answered)}');

        // Same again on the way out, for whatever writes the socket.
        return response.change(body: answered);
      };
    };
  }

  /// The headers, with the ones that carry credentials replaced.
  Map<String, String> _safeHeaders(Map<String, String> headers) => {
        for (final entry in headers.entries)
          entry.key: redacted.contains(entry.key.toLowerCase())
              ? '<redacted>'
              : entry.value,
      };

  String _clip(String body) =>
      body.length <= limit ? body : '${body.substring(0, limit)}… (clipped)';
}
