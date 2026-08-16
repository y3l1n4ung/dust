import 'dart:convert';
import 'dart:typed_data';

import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../response/rejection.dart';
import 'body_reader.dart';
import 'extractable.dart';

/// Reads the body as raw bytes, with no media type requirement.
final class RawBodyExtractable implements FromRequest<Uint8List> {
  /// Reads the raw body.
  const RawBodyExtractable({this.limit = defaultBodyLimit});

  /// The maximum body size, in bytes.
  final int limit;

  @override
  Future<Result<Uint8List, Rejection>> extract(Request request) =>
      readBody(request, limit: limit);
}

/// Reads the body as UTF-8 text.
///
/// The `charset` parameter on `content-type` is ignored; anything that is not
/// UTF-8 is a 400.
final class TextBodyExtractable implements FromRequest<String> {
  /// Reads the body as text.
  const TextBodyExtractable({this.limit = defaultBodyLimit});

  /// The maximum body size, in bytes.
  final int limit;

  @override
  Future<Result<String, Rejection>> extract(Request request) async {
    switch (await readBody(request, limit: limit)) {
      case Err(:final error):
        return Err(error);
      case Ok(:final value):
        try {
          return Ok(utf8.decode(value));
        } on FormatException {
          return const Err(Rejection.badRequest('body is not valid UTF-8'));
        }
    }
  }
}

/// Hands the body to the handler as an unread stream.
///
/// The limit cannot be enforced here, so a streaming handler owns that policy.
final class StreamBodyExtractable implements FromRequest<Stream<List<int>>> {
  /// Passes the body stream through.
  const StreamBodyExtractable();

  @override
  Future<Result<Stream<List<int>>, Rejection>> extract(Request request) async {
    return Ok(request.read());
  }
}
