import 'dart:convert';
import 'dart:typed_data';

import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../request/request_parts.dart';
import '../response/rejection.dart';
import '../response/validation.dart';
import 'body_reader.dart';
import 'extractable.dart';

/// Decodes a JSON object body into [T].
///
/// [deserialize] is the free function a `Deserialize` derive generates, so a
/// model written for Dust needs nothing extra:
///
/// ```dart
/// const createTodo = JsonExtractable<CreateTodo>(CreateTodo.deserialize);
/// ```
///
/// Rejects with 415 for a non-JSON `content-type`, 400 for malformed syntax,
/// and 422 when the JSON parses but [deserialize] cannot build a [T] from it.
final class JsonExtractable<T> implements FromRequest<T> {
  /// Decodes the body with [deserialize].
  const JsonExtractable(this.deserialize, {this.limit = defaultBodyLimit});

  /// Builds the value from a decoded JSON object.
  final T Function(Map<String, Object?> json) deserialize;

  /// The maximum body size, in bytes.
  final int limit;

  @override
  Future<Result<T, Rejection>> extract(Request request) async {
    final Object? value;
    switch (await _decodeJsonBody(request, limit)) {
      case Err(:final error):
        return Err(error);
      case Ok(value: final decoded):
        value = decoded;
    }

    if (value is! Map<String, Object?>) {
      return const Err(
        Rejection.unprocessable({}, message: 'expected a JSON object'),
      );
    }

    try {
      return Ok(deserialize(value));
    } on Object catch (error) {
      return Err(decodeRejection(error, subject: 'JSON body'));
    }
  }
}

/// Decodes a JSON array body into `List<T>`.
final class JsonListExtractable<T> implements FromRequest<List<T>> {
  /// Decodes each element with [deserialize].
  const JsonListExtractable(this.deserialize, {this.limit = defaultBodyLimit});

  /// Builds one element from a decoded JSON object.
  final T Function(Map<String, Object?> json) deserialize;

  /// The maximum body size, in bytes.
  final int limit;

  @override
  Future<Result<List<T>, Rejection>> extract(Request request) async {
    final Object? value;
    switch (await _decodeJsonBody(request, limit)) {
      case Err(:final error):
        return Err(error);
      case Ok(value: final decoded):
        value = decoded;
    }

    if (value is! List) {
      return const Err(
        Rejection.unprocessable({}, message: 'expected a JSON array'),
      );
    }

    try {
      return Ok([
        for (final element in value)
          deserialize(element! as Map<String, Object?>),
      ]);
    } on Object catch (error) {
      return Err(decodeRejection(error, subject: 'JSON array'));
    }
  }
}

/// Reads and parses a JSON body, leaving the shape check to the caller.
Future<Result<Object?, Rejection>> _decodeJsonBody(
  Request request,
  int limit,
) async {
  if (!isJsonMediaType(RequestParts.of(request).mediaType)) {
    return const Err(
      Rejection.unsupportedMediaType('expected application/json'),
    );
  }

  final Uint8List raw;
  switch (await readBody(request, limit: limit)) {
    case Err(:final error):
      return Err(error);
    case Ok(:final value):
      raw = value;
  }

  if (raw.isEmpty) {
    return const Err(Rejection.badRequest('request body is empty'));
  }

  try {
    return Ok(jsonDecode(utf8.decode(raw)));
  } on FormatException catch (error) {
    return Err(Rejection.badRequest('malformed JSON: ${error.message}'));
  }
}
