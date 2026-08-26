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
/// upstream service survives instead of being replaced — but only when it
/// looks like an id. An inbound value is client-controlled and ends up in the
/// response, the access log, and whatever reads that log, so an unchecked one
/// hands a client several kilobytes of your logs per request. Anything longer
/// than [maxLength] or carrying a character outside `A-Z a-z 0-9 - _ .` is
/// replaced by a generated id rather than echoed.
///
/// ```dart
/// final app = Router()..layer(const RequestId());
/// ```
final class RequestId implements Layer {
  /// Reads or assigns an id, under [header].
  const RequestId({
    this.header = 'x-request-id',
    this.generate,
    this.maxLength = 128,
  });

  /// The header carrying the id, in and out.
  final String header;

  /// Builds an id when the request arrived without one.
  ///
  /// Defaults to a version 4 UUID from `package:uuid`, so ids are unique
  /// across processes and machines rather than only within one isolate.
  final String Function()? generate;

  /// The longest inbound id that is echoed rather than replaced.
  ///
  /// 128 is comfortably above every id anyone actually sends — a UUID is 36
  /// characters, a W3C trace id 32 — and far below what a client can use to
  /// inflate a log line.
  final int maxLength;

  @override
  Middleware toMiddleware() {
    return (Handler inner) {
      return (Request request) async {
        final inbound = request.headers[header];
        final id = _acceptableId(inbound, maxLength)
            ? inbound!
            : (generate ?? _randomId)();
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

/// Whether an inbound id is safe to echo and log.
///
/// Not a correctness check — there is no format every proxy agrees on — but a
/// bound on what a client can put into a header this layer copies into the
/// response and the access log.
bool _acceptableId(String? value, int maxLength) {
  if (value == null || value.isEmpty || value.length > maxLength) return false;
  for (final unit in value.codeUnits) {
    final ok = (unit >= 0x30 && unit <= 0x39) || // 0-9
        (unit >= 0x41 && unit <= 0x5a) || // A-Z
        (unit >= 0x61 && unit <= 0x7a) || // a-z
        unit == 0x2d || // -
        unit == 0x5f || // _
        unit == 0x2e; // .
    if (!ok) return false;
  }
  return true;
}
