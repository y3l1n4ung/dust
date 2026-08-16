import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../request/coercion.dart';
import '../request/request_parts.dart';
import '../response/rejection.dart';
import 'extractable.dart';

/// Extracts one router-captured path parameter, coerced to [T].
///
/// `@Path() String id` lowers to `const PathExtractable<String>('id')`; the key
/// defaults to the Dart parameter name and `@Path('todo_id')` overrides it.
///
/// The matcher captures the raw segment, so decoding happens here:
/// `/files/a%20b` gives `a b`, not `a%20b`.
final class PathExtractable<T> implements FromRequestParts<T> {
  /// Reads the path parameter named [key].
  const PathExtractable(this.key);

  /// The path parameter name, matching a `{key}` segment in the route.
  final String key;

  @override
  Future<Result<T, Rejection>> extract(Request request) async {
    final raw = pathParametersOf(request)[key];
    if (raw == null) {
      return Err(Rejection.badRequest('path parameter "$key" is missing'));
    }

    final String decoded;
    try {
      decoded = Uri.decodeComponent(raw);
    } on ArgumentError {
      return Err(
        Rejection.badRequest(
          'path parameter "$key" is not valid percent-encoding',
        ),
      );
    }

    return coerce<T>(decoded, source: 'path parameter "$key"');
  }
}
