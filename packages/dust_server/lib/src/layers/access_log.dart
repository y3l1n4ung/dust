import 'package:shelf/shelf.dart';

import '../request/request_parts.dart';
import '../router/middleware.dart';
import 'request_id.dart';

/// One served request, as the application should record it.
final class AccessRecord {
  /// Describes a finished request.
  const AccessRecord({
    required this.method,
    required this.path,
    required this.status,
    required this.duration,
    this.requestId,
    this.matchedRoute,
  });

  /// The request method.
  final String method;

  /// The path as requested.
  final String path;

  /// The status that went back.
  final int status;

  /// How long the handler took.
  final Duration duration;

  /// The id from a [RequestId] layer, when one ran.
  final String? requestId;

  /// The route pattern that served it — `/orders/{id}` — or `null` for a 404.
  ///
  /// This, not [path], is what a metric or a dashboard groups by. `path` is one
  /// value per order; the pattern is one value per endpoint, which is the
  /// difference between a chart and a cardinality explosion.
  final String? matchedRoute;

  @override
  String toString() {
    final id = requestId == null ? '' : ' [$requestId]';
    return '$method $path $status ${duration.inMilliseconds}ms$id';
  }
}

/// Reports every served request to [onRecord].
///
/// Nothing is printed: what a log line looks like, and where it goes, is the
/// application's decision.
///
/// ```dart
/// final app = Router()
///   ..layer(const RequestId())
///   ..layer(AccessLog(logger.info));
/// ```
final class AccessLog implements Layer {
  /// Reports each request to [onRecord].
  const AccessLog(this.onRecord);

  /// Called once per request, after the response is chosen.
  final void Function(AccessRecord record) onRecord;

  @override
  Middleware toMiddleware() {
    return (Handler inner) {
      return (Request request) async {
        final stopwatch = Stopwatch()..start();

        // The router fills this on the way through. A layer wraps the matcher,
        // so it cannot read the matched route off its own request.
        final matched = MatchedRouteSlot();
        final logged = request.change(
          context: {matchedRouteSlotKey: matched},
        );

        final response = await inner(logged);
        stopwatch.stop();

        onRecord(
          AccessRecord(
            method: request.method,
            path: '/${request.url.path}',
            status: response.statusCode,
            duration: stopwatch.elapsed,
            requestId: requestIdOf(request),
            matchedRoute: matched.route,
          ),
        );
        return response;
      };
    };
  }
}
