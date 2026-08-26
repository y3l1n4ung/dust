import 'dart:convert';
import 'dart:io';

import 'package:dust_server/server.dart';

/// Changing what a failure looks like on the wire.
///
/// A `Rejection` encodes as `{"error": ..., "fields": ...}`. That is a choice,
/// not a law — an API with a published contract, or one behind a gateway that
/// expects RFC 9457 `application/problem+json`, needs a different shape.
///
/// Two places to change it, and they are not the same job:
///
/// * **A layer**, as here. Every failure is reshaped, including the ones raised
///   by extractors before your handler ran — a 400 from a bad path parameter,
///   a 405 from the router. Nothing is missed, which is the reason to do it here.
/// * **A handler**, by taking the `Result` from `request.extract(...)` and
///   returning your own type. Use that when one endpoint is special, not when
///   the whole API is.
///
/// Run it with `dart run example/customize_rejection.dart`:
///
/// ```bash
/// curl -s localhost:8080/orders/41
/// curl -s localhost:8080/orders/abc   # problem+json, not the default shape
/// curl -s localhost:8080/nothing      # the router's own 404, reshaped too
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  return Router()
    ..layer(const ProblemDetails())
    ..route('/orders/{id}', get(readOrder));
}

/// `GET /orders/{id}`
Future<Map<String, Object?>> readOrder(Request request) async => {
      'id': await request.path<int>('id'),
    };

/// Rewrites every failure as RFC 9457 `application/problem+json`.
final class ProblemDetails implements Layer {
  /// Reshapes responses at or above [from].
  const ProblemDetails({this.from = 400});

  /// The lowest status treated as a failure.
  final int from;

  @override
  Middleware toMiddleware() {
    return (Handler inner) {
      return (Request request) async {
        final response = await inner(request);
        if (response.statusCode < from) return response;

        // The body has to be read to be rewritten, and a response body can only
        // be read once — so this returns a new response rather than changing
        // the one it was handed.
        final original = await response.readAsString();
        final decoded = _decode(original);

        return Response(
          response.statusCode,
          body: jsonEncode({
            'type': 'about:blank',
            'title': decoded['error'] ?? original,
            'status': response.statusCode,
            'instance': request.requestedUri.path,
            if (decoded['fields'] case final fields?) 'errors': fields,
          }),
          headers: {
            ...response.headers,
            'content-type': 'application/problem+json',
          },
        );
      };
    };
  }

  /// The rejection body, or an empty map when it was not JSON.
  Map<String, Object?> _decode(String body) {
    try {
      return jsonDecode(body) as Map<String, Object?>;
    } on Object {
      return const {};
    }
  }
}
