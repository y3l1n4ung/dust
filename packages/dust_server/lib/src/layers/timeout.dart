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
