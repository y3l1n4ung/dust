import 'package:shelf/shelf.dart';
import 'package:uuid/uuid.dart';

import '../router/middleware.dart';

/// The context key a request id travels under.
const requestIdContextKey = 'dust_server/request_id';

/// The id given to this request, or `null` when no [RequestId] layer ran.
String? requestIdOf(Request request) =>
    request.context[requestIdContextKey] as String?;

/// Gives every request an id, and echoes it back.
///
/// An inbound `x-request-id` is kept, so an id assigned by a proxy or an
/// upstream service survives instead of being replaced.
///
/// ```dart
/// final app = Router()..layer(const RequestId());
/// ```
final class RequestId implements Layer {
  /// Reads or assigns an id, under [header].
  const RequestId({this.header = 'x-request-id', this.generate});

  /// The header carrying the id, in and out.
  final String header;

  /// Builds an id when the request arrived without one.
  ///
  /// Defaults to a version 4 UUID from `package:uuid`, so ids are unique
  /// across processes and machines rather than only within one isolate.
  final String Function()? generate;

  @override
  Middleware toMiddleware() {
    return (Handler inner) {
      return (Request request) async {
        final id = request.headers[header] ?? (generate ?? _randomId)();
        final response = await inner(
          request.change(context: {requestIdContextKey: id}),
        );
        return response.change(headers: {header: id});
      };
    };
  }
}

const _uuid = Uuid();

String _randomId() => _uuid.v4();
