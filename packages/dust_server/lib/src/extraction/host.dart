import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../request/request_parts.dart';
import '../response/rejection.dart';
import 'extractable.dart';

/// The host a request was addressed to.
///
/// Read for multi-tenant routing, for building an absolute URL back to
/// yourself, or for refusing a `Host` nobody serves.
///
/// **Treat it as untrusted.** A client sends it, and a request that reached
/// your server with `Host: evil.test` says nothing about who owns the name. A
/// URL built from it and put in an email is a redirect anyone can aim. Compare
/// it against a list you control; do not interpolate it.
///
/// Behind a proxy the original host is in `X-Forwarded-Host`, and that is no
/// more trustworthy — a proxy you control may rewrite it, and anything else
/// can forge it. [HostExtractable] reads `Host` only, deliberately: preferring
/// a forwarded header by default would make a spoof the easy path.
final class HostExtractable implements FromRequestParts<String> {
  /// Reads the `Host` header.
  const HostExtractable();

  @override
  Future<Result<String, Rejection>> extract(Request request) async {
    final host = RequestParts.of(request).headers['host'];
    if (host == null || host.isEmpty) {
      // HTTP/1.1 requires it, so its absence is a malformed request rather
      // than a missing optional value.
      return const Err(Rejection.badRequest('the Host header is required'));
    }
    return Ok(host);
  }
}
