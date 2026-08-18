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
/// it is read. A body arriving without one — chunked, which the client chooses
/// — is counted as it flows, because a declared length is a claim.
///
/// The effective limit is the **stricter** of [limit] and any limit configured
/// on the router. See [effectiveBodyLimit].
Future<Result<Uint8List, Rejection>> readBody(
  Request request, {
  int limit = defaultBodyLimit,
}) async {
  final effective = effectiveBodyLimit(request, limit);

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

/// The smaller of [limit] and whatever the router configured.
///
/// Two places set a body limit and neither should be able to loosen the other:
/// `Router(bodyLimit:)` bounds the whole application, and an extractor bounds
/// one route. Taking the minimum means adding either can only ever refuse more.
///
/// It used to be "the router wins", which reached generated code correctly and
/// also quietly *raised* a limit an extractor had deliberately set low — a route
/// that asked for 1 KB accepted whatever the application allowed.
int effectiveBodyLimit(Request request, int limit) {
  final configured = request.context[bodyLimitContextKey];
  return configured is int && configured < limit ? configured : limit;
}

/// Whether [mediaType] is JSON or a `+json` structured syntax suffix.
bool isJsonMediaType(String? mediaType) {
  if (mediaType == null) return false;
  return mediaType == 'application/json' || mediaType.endsWith('+json');
}
