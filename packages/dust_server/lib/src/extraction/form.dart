import 'dart:convert';

import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../request/coercion.dart';
import '../request/request_parts.dart';
import '../response/rejection.dart';
import 'body_reader.dart';
import 'extractable.dart';

/// The decoded fields of a form body.
///
/// One extractor reads the body and `@Field` parameters read from the result,
/// so the body is consumed once no matter how many fields a handler declares.
final class FormMap {
  /// Wraps decoded [fields].
  const FormMap(this.fields);

  /// The decoded field values.
  final Map<String, String> fields;

  /// Reads field [name], coerced to [T].
  ///
  /// A nullable [T] makes the field optional.
  Result<T, Rejection> field<T>(String name) {
    final raw = fields[name];
    if (raw == null) {
      if (null is T) return Ok(null as T);
      return Err(Rejection.badRequest('form field "$name" is required'));
    }
    return coerce<T>(raw, source: 'form field "$name"');
  }
}

/// Decodes an `application/x-www-form-urlencoded` body.
///
/// A `GET` or `HEAD` request has no body, so those read the query string
/// instead and the same handler works for both.
final class FormExtractable implements FromRequest<FormMap> {
  /// Decodes the form body.
  const FormExtractable({this.limit = defaultBodyLimit});

  /// The maximum body size, in bytes.
  final int limit;

  @override
  Future<Result<FormMap, Rejection>> extract(Request request) async {
    final parts = RequestParts.of(request);
    if (parts.method == 'GET' || parts.method == 'HEAD') {
      return Ok(FormMap(request.url.queryParameters));
    }

    if (parts.mediaType != 'application/x-www-form-urlencoded') {
      return const Err(
        Rejection.unsupportedMediaType(
          'expected application/x-www-form-urlencoded',
        ),
      );
    }

    switch (await readBody(request, limit: limit)) {
      case Err(:final error):
        return Err(error);
      case Ok(:final value):
        try {
          return Ok(FormMap(Uri.splitQueryString(utf8.decode(value))));
        } on FormatException catch (error) {
          return Err(
            Rejection.badRequest('malformed form body: ${error.message}'),
          );
        }
    }
  }
}
