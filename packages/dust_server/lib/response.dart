/// Responses: what a handler answers with, and how failures become status
/// codes.
///
/// Import this for a type that turns itself into a response, or for the
/// encoders generated handlers call.
///
/// ```dart
/// import 'package:dust_server/response.dart';
///
/// final class NotFound implements IntoResponse {
///   const NotFound(this.message);
///
///   final String message;
///
///   @override
///   Response intoResponse() => jsonResponse({'error': message}, status: 404);
/// }
/// ```
///
/// Everything here is also exported from `package:dust_server/server.dart`.
library;

export 'package:shelf/shelf.dart' show Response;

export 'src/response/response.dart';
