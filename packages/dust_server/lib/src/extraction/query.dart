import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../request/coercion.dart';
import '../response/rejection.dart';
import '../response/validation.dart';
import 'extractable.dart';

/// Extracts one query value, coerced to [T].
///
/// A nullable [T] makes the value optional; a non-nullable [T] rejects with 400
/// when the key is absent.
final class QueryExtractable<T> implements FromRequestParts<T> {
  /// Reads the query value named [key].
  const QueryExtractable(this.key);

  /// The query key.
  final String key;

  @override
  Future<Result<T, Rejection>> extract(Request request) async {
    final raw = request.url.queryParameters[key];
    if (raw == null) {
      if (null is T) return Ok(null as T);
      return Err(Rejection.badRequest('query "$key" is required'));
    }
    return coerce<T>(raw, source: 'query "$key"');
  }
}

/// Extracts every value for one repeated query key, coerced to [T].
///
/// `Uri.queryParameters` keeps only the last value for a repeated key, so
/// reading `?tag=a&tag=b` needs this instead of [QueryExtractable]. An absent
/// key gives an empty list rather than a rejection.
final class QueryListExtractable<T> implements FromRequestParts<List<T>> {
  /// Reads every value for [key].
  const QueryListExtractable(this.key);

  /// The query key.
  final String key;

  @override
  Future<Result<List<T>, Rejection>> extract(Request request) async {
    final raw = request.url.queryParametersAll[key] ?? const <String>[];
    final values = <T>[];
    for (final value in raw) {
      switch (coerce<T>(value, source: 'query "$key"')) {
        case Err(:final error):
          return Err(error);
        case Ok(value: final coerced):
          values.add(coerced);
      }
    }
    return Ok(values);
  }
}

/// Extracts every query value as a map.
///
/// `@Queries() TodoFilter filter` decodes this map through the type's
/// `Deserialize` implementation; `@Queries() Map<String, String> raw` takes it
/// as-is.
final class QueriesExtractable<T> implements FromRequestParts<T> {
  /// Decodes the query map with [deserialize].
  const QueriesExtractable(this.deserialize);

  /// Builds the value from the collected query pairs.
  final T Function(Map<String, String> queries) deserialize;

  @override
  Future<Result<T, Rejection>> extract(Request request) async {
    try {
      return Ok(deserialize(request.url.queryParameters));
    } on Object catch (error) {
      return Err(decodeRejection(error, subject: 'query parameters'));
    }
  }
}

/// Extracts the raw, undecoded query string.
final class RawQueryExtractable implements FromRequestParts<String?> {
  /// Reads the query component of the request URI.
  const RawQueryExtractable();

  @override
  Future<Result<String?, Rejection>> extract(Request request) async {
    final query = request.url.query;
    return Ok(query.isEmpty ? null : query);
  }
}
