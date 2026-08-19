import 'package:shelf/shelf.dart';

/// A value that can turn itself into an HTTP [Response].
///
/// Handler return types implementing this are written to the wire by calling
/// [intoResponse]. Dust also detects the method structurally, so an unrelated
/// class exposing `Response intoResponse()` works the same way.
///
/// ```dart
/// class NotFound implements IntoResponse {
///   const NotFound(this.message);
///
///   final String message;
///
///   @override
///   Response intoResponse() => jsonResponse({'error': message}, status: 404);
/// }
/// ```
abstract interface class IntoResponse {
  /// Builds the response for this value.
  Response intoResponse();
}
