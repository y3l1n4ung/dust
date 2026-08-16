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
