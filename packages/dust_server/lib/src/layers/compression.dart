import 'dart:io' show GZipCodec;

import 'package:shelf/shelf.dart';

import '../request/request_parts.dart';
import '../router/middleware.dart';

/// Content types worth compressing.
///
/// Text compresses; a JPEG does not. Spending CPU to make an already-compressed
/// body 2% larger is the usual cost of a naive compressor, so the default list
/// names what actually benefits.
const defaultCompressibleTypes = <String>{
  'application/json',
  'application/javascript',
  'application/xml',
  'application/xhtml+xml',
  'image/svg+xml',
  'text/css',
  'text/csv',
  'text/event-stream',
  'text/html',
  'text/javascript',
  'text/plain',
  'text/xml',
};

/// Compresses responses a client said it can decompress.
///
/// ```dart
/// final app = Router()
///   ..layer(const Compression())
///   ..merge(routes);
/// ```
///
/// Only gzip: it is the encoding every client has supported for two decades,
/// and brotli would mean a dependency for a few percent. A client that asks
/// for nothing, or asks only for something else, gets the body uncompressed.
///
/// A response is left alone when any of these is true, because compressing it
/// would cost more than it saves or would break something:
///
/// * the client did not offer gzip;
/// * the body is already encoded — re-encoding is how a double-gzipped
///   response happens, which no client will unwrap twice;
/// * the type is not in [types], so it is probably compressed already;
/// * a known length is below [minimumBytes], where the gzip header alone can
///   make the answer bigger.
///
/// `Vary: Accept-Encoding` is always added, compressed or not: without it a
/// shared cache can hand a gzipped body to a client that never asked for one.
final class Compression implements Layer {
  /// Compresses text-ish responses over [minimumBytes].
  const Compression({
    this.minimumBytes = 1024,
    this.types = defaultCompressibleTypes,
    this.level = 6,
  });

  /// Below this many bytes, compression is not worth the header.
  ///
  /// Only applied when the length is known; a streamed body of unknown length
  /// is compressed, since waiting to find out would mean buffering it.
  final int minimumBytes;

  /// Which content types are worth compressing.
  final Set<String> types;

  /// The gzip level, 1 (fastest) to 9 (smallest).
  final int level;

  @override
  Middleware toMiddleware() {
    return (Handler inner) {
      return (Request request) async {
        final accepts = _acceptsGzip(
          RequestParts.of(request).headers['accept-encoding'],
        );

        final response = await inner(request);
        // `Vary` goes on either way, so a cache never reuses one client's
        // encoding for another's request.
        final varied = response.change(
          headers: {'vary': _vary(response.headers['vary'])},
        );

        if (!accepts || !_worthCompressing(varied)) return varied;

        return varied.change(
          headers: {
            'content-encoding': 'gzip',
            // The encoded length is not the declared one, and computing it
            // would mean buffering the whole body.
            'content-length': null,
          },
          body: GZipCodec(level: level)
              .encoder
              .bind(varied.read().map(List<int>.from)),
        );
      };
    };
  }

  /// Whether [header] offers gzip with a weight above zero.
  ///
  /// `gzip;q=0` is a client saying explicitly that it does not want gzip, and
  /// reading it as an offer is a classic way to break one stubborn client.
  static bool _acceptsGzip(String? header) {
    if (header == null) return false;

    for (final part in header.split(',')) {
      final pieces = part.trim().split(';');
      final coding = pieces.first.trim().toLowerCase();
      if (coding != 'gzip' && coding != '*') continue;

      for (final parameter in pieces.skip(1)) {
        final trimmed = parameter.trim().toLowerCase();
        if (!trimmed.startsWith('q=')) continue;
        final weight = double.tryParse(trimmed.substring(2)) ?? 1;
        if (weight <= 0) return false;
      }
      return true;
    }
    return false;
  }

  bool _worthCompressing(Response response) {
    if (response.headers.containsKey('content-encoding')) return false;

    final type = response.headers['content-type']?.split(';').first.trim();
    if (type == null || !types.contains(type.toLowerCase())) return false;

    final length = response.contentLength;
    if (length != null && length < minimumBytes) return false;

    return true;
  }

  /// `Accept-Encoding` added to whatever the response already varies on.
  static String _vary(String? existing) {
    if (existing == null || existing.isEmpty) return 'Accept-Encoding';

    final already = existing
        .split(',')
        .map((value) => value.trim().toLowerCase())
        .contains('accept-encoding');
    return already ? existing : '$existing, Accept-Encoding';
  }
}
