import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../request/coercion.dart';
import '../response/rejection.dart';
import 'extractable.dart';

/// Extracts one header, coerced to [T].
///
/// Header names are matched case-insensitively. A nullable [T] makes the header
/// optional.
final class HeaderExtractable<T> implements FromRequestParts<T> {
  /// Reads the header named [name].
  const HeaderExtractable(this.name);

  /// The header name.
  final String name;

  @override
  Future<Result<T, Rejection>> extract(Request request) async {
    final raw = request.headers[name.toLowerCase()];
    if (raw == null) {
      if (null is T) return Ok(null as T);
      return Err(Rejection.badRequest('header "$name" is required'));
    }
    return coerce<T>(raw, source: 'header "$name"');
  }
}

/// Extracts every header as a map with lower-case keys.
///
/// The keys are lower-cased on the way out because shelf's own header map is
/// case-insensitive and a plain copy of it would not be.
final class HeaderMapExtractable
    implements FromRequestParts<Map<String, String>> {
  /// Reads all headers.
  const HeaderMapExtractable();

  @override
  Future<Result<Map<String, String>, Rejection>> extract(
    Request request,
  ) async {
    return Ok(
      Map<String, String>.unmodifiable({
        for (final entry in request.headers.entries)
          entry.key.toLowerCase(): entry.value,
      }),
    );
  }
}
