import 'dart:convert';
import 'dart:io' show HeaderValue;
import 'dart:typed_data';

import 'package:dust_dart/fp.dart';
import 'package:mime/mime.dart';
import 'package:shelf/shelf.dart';

import '../request/coercion.dart';
import '../request/request_parts.dart';
import '../response/rejection.dart';
import 'body_reader.dart';
import 'extractable.dart';

/// One decoded part of a multipart body.
final class UploadedFile {
  /// Creates one part.
  const UploadedFile({
    required this.name,
    required this.bytes,
    this.filename,
    this.contentType,
  });

  /// The form field name.
  final String name;

  /// The raw part content.
  final Uint8List bytes;

  /// The client-supplied file name, when the part was a file.
  final String? filename;

  /// The part's own `content-type`, when it declared one.
  final String? contentType;

  /// The part content decoded as UTF-8 text.
  String get asText => utf8.decode(bytes);
}

/// The decoded parts of a `multipart/form-data` body.
final class MultipartForm {
  /// Wraps decoded [parts].
  const MultipartForm(this.parts);

  /// Every part, keyed by form field name. Repeated names keep the last part.
  final Map<String, UploadedFile> parts;

  /// Reads part [name] as a file.
  Result<UploadedFile, Rejection> file(String name) {
    final part = parts[name];
    if (part == null) {
      return Err(Rejection.badRequest('multipart part "$name" is required'));
    }
    return Ok(part);
  }

  /// Reads part [name] as text, coerced to [T].
  Result<T, Rejection> field<T>(String name) {
    final part = parts[name];
    if (part == null) {
      if (null is T) return Ok(null as T);
      return Err(Rejection.badRequest('multipart part "$name" is required'));
    }
    return coerce<T>(part.asText, source: 'multipart part "$name"');
  }
}

/// Decodes a `multipart/form-data` body.
///
/// Every part is buffered rather than streamed, because several `@Part`
/// parameters have to read from one pass over the body. [limit] bounds the
/// whole body, not each part, so it is not a good fit for large uploads.
final class MultipartExtractable implements FromRequest<MultipartForm> {
  /// Decodes the multipart body.
  const MultipartExtractable({this.limit = defaultBodyLimit});

  /// The maximum body size, in bytes.
  final int limit;

  @override
  Future<Result<MultipartForm, Rejection>> extract(Request request) async {
    if (RequestParts.of(request).mediaType != 'multipart/form-data') {
      return const Err(
        Rejection.unsupportedMediaType('expected multipart/form-data'),
      );
    }

    final boundary =
        RequestParts.of(request).contentType?.parameters['boundary'];
    if (boundary == null) {
      return const Err(
        Rejection.badRequest('multipart body declares no boundary'),
      );
    }

    switch (await readBody(request, limit: limit)) {
      case Err(:final error):
        return Err(error);
      case Ok(:final value):
        try {
          return Ok(MultipartForm(await _readParts(value, boundary)));
        } on Object catch (error) {
          return Err(Rejection.badRequest('malformed multipart body: $error'));
        }
    }
  }

  Future<Map<String, UploadedFile>> _readParts(
    Uint8List raw,
    String boundary,
  ) async {
    final decoded = <String, UploadedFile>{};
    final stream = Stream<List<int>>.value(raw)
        .transform(MimeMultipartTransformer(boundary));

    await for (final part in stream) {
      final disposition =
          HeaderValue.parse(part.headers['content-disposition'] ?? '');
      final name = disposition.parameters['name'];
      if (name == null) continue;

      final builder = BytesBuilder(copy: false);
      await for (final chunk in part) {
        builder.add(chunk);
      }

      decoded[name] = UploadedFile(
        name: name,
        bytes: builder.takeBytes(),
        filename: disposition.parameters['filename'],
        contentType: part.headers['content-type'],
      );
    }
    return decoded;
  }
}
