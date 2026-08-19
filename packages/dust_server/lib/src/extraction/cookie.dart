import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../request/coercion.dart';
import '../request/request_parts.dart';
import '../response/rejection.dart';
import 'extractable.dart';

/// The cookies one request carried.
///
/// A browser sends every cookie for the origin in one header, so reading one
/// means picking it out of a list. Parsing it once and handing back a map is
/// what keeps that out of every extractor that needs a session.
final class CookieJar {
  /// Wraps already-parsed [values].
  const CookieJar(this.values);

  /// Reads the `cookie` header of [request].
  ///
  /// Unparseable pairs are skipped rather than failing the request: a stray
  /// cookie set by something else on the domain is not this application's
  /// problem, and refusing the request over one would be.
  factory CookieJar.of(Request request) {
    final header = RequestParts.of(request).headers['cookie'];
    if (header == null) return const CookieJar({});

    final values = <String, String>{};
    for (final pair in header.split(';')) {
      final separator = pair.indexOf('=');
      if (separator < 1) continue;

      final name = pair.substring(0, separator).trim();
      if (name.isEmpty) continue;

      var value = pair.substring(separator + 1).trim();
      if (value.length > 1 && value.startsWith('"') && value.endsWith('"')) {
        value = value.substring(1, value.length - 1);
      }
      values.putIfAbsent(name, () => value);
    }
    return CookieJar(values);
  }

  /// The cookies, by name.
  final Map<String, String> values;

  /// The cookie named [name], or `null`.
  String? operator [](String name) => values[name];

  /// Whether [name] was sent.
  bool contains(String name) => values.containsKey(name);

  /// The names that were sent.
  Iterable<String> get names => values.keys;
}

/// Extracts one cookie, coerced to [T].
///
/// A nullable [T] makes it optional; a non-nullable one rejects with 400 when
/// the cookie is absent, which matches how a query is read.
final class CookieExtractable<T> implements FromRequestParts<T> {
  /// Reads the cookie named [name].
  const CookieExtractable(this.name);

  /// The cookie name, matched exactly — cookie names are case-sensitive.
  final String name;

  @override
  Future<Result<T, Rejection>> extract(Request request) async {
    final raw = CookieJar.of(request)[name];
    if (raw == null) {
      if (null is T) return Ok(null as T);
      return Err(Rejection.badRequest('cookie "$name" is required'));
    }
    return coerce<T>(raw, source: 'cookie "$name"');
  }
}

/// Extracts every cookie at once.
final class CookieJarExtractable implements FromRequestParts<CookieJar> {
  /// Reads all cookies.
  const CookieJarExtractable();

  @override
  Future<Result<CookieJar, Rejection>> extract(Request request) async =>
      Ok(CookieJar.of(request));
}
