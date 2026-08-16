/// Extractors: turning a request into the values a handler declares.
///
/// Import this when writing a custom extractor, or when calling the built-ins
/// by hand rather than through generated code.
///
/// ```dart
/// import 'package:dust_server/extraction.dart';
///
/// final class BearerAuth implements FromRequestParts<AuthUser> {
///   const BearerAuth();
///
///   @override
///   Future<Result<AuthUser, Rejection>> extract(Request request) async { ... }
/// }
/// ```
///
/// Everything here is also exported from `package:dust_server/server.dart`.
library;

export 'package:dust_dart/fp.dart';
export 'package:shelf/shelf.dart' show Request;

export 'src/extraction/extraction.dart';
export 'src/request/request.dart';
