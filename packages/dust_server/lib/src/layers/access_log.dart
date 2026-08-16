import 'package:shelf/shelf.dart';

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
        final response = await inner(request);
        stopwatch.stop();

        onRecord(
          AccessRecord(
            method: request.method,
            path: '/${request.url.path}',
            status: response.statusCode,
            duration: stopwatch.elapsed,
            requestId: requestIdOf(request),
          ),
        );
        return response;
      };
    };
  }
}
