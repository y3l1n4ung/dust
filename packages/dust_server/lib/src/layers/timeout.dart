import 'dart:async';

import 'package:shelf/shelf.dart';

import '../response/rejection.dart';
import '../router/middleware.dart';

/// Gives every request below it a deadline.
///
/// Without one, a handler that never completes holds its connection forever.
/// The body limit bounds how much a request can send; this bounds how long it
/// can take.
///
/// > **It bounds producing the response, not sending it.** The budget covers the
/// > handler returning a [Response]. A handler that returns immediately with a
/// > streamed body — an event stream, a large download — has already satisfied
/// > it, and the stream then runs for as long as it likes. Adding this layer
/// > above an SSE endpoint protects nothing there, and the connection is held
/// > until the stream ends or the client leaves. Bound those in the stream
/// > itself, with `take`, a timeout on the source, or a keep-alive the client
/// > answers.
///
/// ```dart
/// final app = Router()
///   ..layer(const RequestTimeout(Duration(seconds: 30)))
///   ..merge(routes);
/// ```
final class RequestTimeout implements Layer {
  /// Fails a request that takes longer than [budget].
  const RequestTimeout(this.budget, {this.onTimeout});

  /// How long a handler may take.
  final Duration budget;

  /// Called when the budget runs out, before the 503 goes back.
  ///
  /// The handler keeps running: Dart cannot cancel it. This is the hook for
  /// recording that it happened.
  final void Function(Request request)? onTimeout;

  @override
  Middleware toMiddleware() {
    return (Handler inner) {
      return (Request request) async {
        try {
          return await Future.value(inner(request)).timeout(budget);
        } on TimeoutException {
          onTimeout?.call(request);
          return Rejection.status(
            503,
            'the request did not complete within '
            '${budget.inMilliseconds}ms',
          ).intoResponse();
        }
      };
    };
  }
}
