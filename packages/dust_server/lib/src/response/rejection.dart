import 'dart:convert';

import 'package:shelf/shelf.dart';

import 'into_response.dart';

/// A typed extraction failure.
///
/// Body failures keep their own status codes rather than collapsing into one
/// 400: 415 for the wrong media type, 400 for malformed syntax, 422 for a
/// well-formed body of the wrong shape, 413 past the size limit.
///
/// ```dart
/// return const Err(Rejection.unauthorized('missing bearer token'));
/// ```
final class Rejection implements IntoResponse {
  const Rejection._(
    this.status,
    this.message, [
    this.fields = const {},
    this.challenge,
  ]);

  /// A 400 rejection, used for malformed syntax and failed coercion.
  const Rejection.badRequest(String message) : this._(400, message);

  /// A 401 rejection.
  ///
  /// [challenge] becomes the `WWW-Authenticate` header, which the HTTP
  /// specification requires on a 401. It defaults to `Bearer`; pass another
  /// scheme when the extractor authenticates differently.
  const Rejection.unauthorized(String message, {String challenge = 'Bearer'})
      : this._(401, message, const {}, challenge);

  /// A 403 rejection.
  const Rejection.forbidden(String message) : this._(403, message);

  /// A 404 rejection.
  const Rejection.notFound(String message) : this._(404, message);

  /// A 405 rejection.
  const Rejection.methodNotAllowed(String message) : this._(405, message);

  /// A 409 rejection.
  const Rejection.conflict(String message) : this._(409, message);

  /// A 413 rejection, used when a body exceeds the configured limit.
  const Rejection.payloadTooLarge(String message) : this._(413, message);

  /// A 415 rejection, used when `content-type` does not match the extractor.
  const Rejection.unsupportedMediaType(String message) : this._(415, message);

  /// A 422 rejection carrying per-field errors.
  ///
  /// Both a shape mismatch during decoding and a failed `Validate()` constraint
  /// land here, so clients see one error format.
  const Rejection.unprocessable(
    Map<String, List<String>> fields, {
    String message = 'Unprocessable entity',
  }) : this._(422, message, fields);

  /// A 500 rejection.
  const Rejection.internal([String message = 'Internal server error'])
      : this._(500, message);

  /// A rejection with an explicit status code.
  const Rejection.status(int status, String message) : this._(status, message);

  /// The HTTP status code this rejection encodes to.
  final int status;

  /// The human-readable message.
  final String message;

  /// Per-field errors, empty unless this is a 422.
  final Map<String, List<String>> fields;

  /// The `WWW-Authenticate` challenge, set on a 401 only.
  final String? challenge;

  @override
  Response intoResponse() {
    final body = <String, Object?>{
      'error': message,
      if (fields.isNotEmpty) 'fields': fields,
    };
    final scheme = challenge;
    return Response(
      status,
      body: jsonEncode(body),
      headers: {
        'content-type': 'application/json',
        if (scheme != null) 'www-authenticate': _headerSafe(scheme),
      },
    );
  }

  /// Strips what cannot appear in a header value.
  ///
  /// [challenge] is the only part of a rejection that reaches a header rather
  /// than the JSON body, so it is the only one where a newline would end the
  /// header and start another. An extractor that builds its challenge from
  /// anything a client influenced — a realm named after a tenant, say — would
  /// otherwise be a response-splitting hole. The body needs no such care:
  /// `jsonEncode` escapes it.
  static String _headerSafe(String value) {
    return value.replaceAll(RegExp(r'[\r\n\x00]'), '');
  }

  @override
  String toString() => 'Rejection($status, $message)';
}
