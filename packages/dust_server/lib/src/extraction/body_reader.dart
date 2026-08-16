import 'dart:typed_data';

import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../request/request_parts.dart';
import '../response/rejection.dart';

/// The default maximum body size, in bytes.
const int defaultBodyLimit = 1024 * 1024;

/// The context key carrying the active body limit.
///
/// `Router.root` installs it so a limit configured at the composition site
/// reaches body extractors whose limit was fixed at build time.
const bodyLimitContextKey = 'dust_server/body_limit';

/// Reads the whole request body, rejecting with 413 past [limit].
///
/// `content-length` is checked first so an oversized upload is refused before
/// it is read. A limit in the request context wins over [limit], which is how
/// `Router.root(bodyLimit:)` reaches generated code.
Future<Result<Uint8List, Rejection>> readBody(
  Request request, {
  int limit = defaultBodyLimit,
}) async {
  final configured = request.context[bodyLimitContextKey];
  final effective = configured is int ? configured : limit;

  Rejection tooLarge() =>
      Rejection.payloadTooLarge('body exceeds $effective bytes');

  final declared = RequestParts.of(request).contentLength;
  if (declared != null && declared > effective) return Err(tooLarge());

  final builder = BytesBuilder(copy: false);
  await for (final chunk in request.read()) {
    builder.add(chunk);
    if (builder.length > effective) return Err(tooLarge());
  }
  return Ok(builder.takeBytes());
}

/// Whether [mediaType] is JSON or a `+json` structured syntax suffix.
bool isJsonMediaType(String? mediaType) {
  if (mediaType == null) return false;
  return mediaType == 'application/json' || mediaType.endsWith('+json');
}
