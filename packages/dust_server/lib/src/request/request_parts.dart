import 'package:http_parser/http_parser.dart';
import 'package:shelf/shelf.dart';

/// The non-body half of a request.
///
/// Extractors that never read the body use this instead of touching
/// [Request.read], which may only be called once.
final class RequestParts {
  const RequestParts._(this.method, this.requestedUri, this.url, this.headers,
      this.context, this.pathParameters);

  /// Reads the parts of [request].
  factory RequestParts.of(Request request) {
    return RequestParts._(
      request.method,
      request.requestedUri,
      request.url,
      request.headers,
      request.context,
      pathParametersOf(request),
    );
  }

  /// The uppercase HTTP method.
  final String method;

  /// The full URI as received, including scheme and host.
  final Uri requestedUri;

  /// The URI relative to the mount point.
  final Uri url;

  /// Request headers, with lower-case keys.
  final Map<String, String> headers;

  /// The shelf request context, which middleware uses to pass values along.
  final Map<String, Object> context;

  /// Path parameters captured by the router.
  final Map<String, String> pathParameters;

  /// The parsed `content-type`, without parameters, or `null` when absent.
  ///
  /// Parsing is `http_parser`'s, so a header carrying parameters, odd casing,
  /// or quoted values is read the same way the rest of the Dart HTTP stack
  /// reads it.
  String? get mediaType => contentType?.mimeType;

  /// The full `content-type`, parameters included, or `null` when absent or
  /// unparseable.
  MediaType? get contentType {
    final raw = headers['content-type'];
    if (raw == null) return null;
    try {
      return MediaType.parse(raw);
    } on FormatException {
      return null;
    }
  }

  /// The `content-length` header as an integer, or `null` when absent or
  /// malformed.
  int? get contentLength => int.tryParse(headers['content-length'] ?? '');
}

/// The context key captured path parameters travel under.
const pathParametersKey = 'dust_server/path_parameters';

/// The context key the matched route pattern travels under.
const matchedPathKey = 'dust_server/matched_path';

/// The context key a [MatchedRouteSlot] travels under.
const matchedRouteSlotKey = 'dust_server/matched_route_slot';

/// A box a layer passes down so the router can report back what it matched.
///
/// A layer wraps the matcher, so it sees the request on the way *in*, before
/// any route has been chosen — reading the matched path from its own request
/// would always find nothing. Handing down a slot the router fills is what
/// lets a layer label a request by route on the way out.
final class MatchedRouteSlot {
  /// Creates an empty slot.
  MatchedRouteSlot();

  /// The route that matched, once one has.
  String? route;
}

/// Fills the slot [request] carries, when it carries one.
void reportMatchedRoute(Request request, String route) {
  final slot = request.context[matchedRouteSlotKey];
  if (slot is MatchedRouteSlot) slot.route = route;
}

/// Reads router-captured path parameters from [request].
///
/// Returns an empty map when the request did not pass through a router, which
/// keeps extractors testable without mounting one.
Map<String, String> pathParametersOf(Request request) {
  final raw = request.context[pathParametersKey];
  if (raw is Map<String, String>) return raw;
  if (raw is Map) return raw.map((k, v) => MapEntry('$k', '$v'));
  return const {};
}

/// The route pattern that matched [request], or `null` when none did.
///
/// The **pattern**, not the path: `/todos/{id}` rather than `/todos/7`. That
/// is what a span name, a metric label, or a log line should carry — naming
/// them after the path makes every id its own operation, and a dashboard of a
/// million one-request operations says nothing.
///
/// `null` for a request that reached the fallback, or one built in a test
/// without a router, because no route matched it.
String? matchedPathOf(Request request) {
  final raw = request.context[matchedPathKey];
  return raw is String ? raw : null;
}
