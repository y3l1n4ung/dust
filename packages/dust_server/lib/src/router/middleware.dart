import 'package:shelf/shelf.dart';

/// Middleware that can be written as a const expression.
///
/// `middleware:` on `@Controller` and on a verb only accepts const values, so
/// there has to be a way to get from one of those to a shelf [Middleware].
/// Dust emits `const RateLimit(perMinute: 30).toMiddleware()`. The name follows
/// axum's `Layer`, and `Router.layer` takes either this or a bare shelf
/// [Middleware].
///
/// ```dart
/// final class RateLimit implements Layer {
///   const RateLimit({required this.perMinute});
///
///   final int perMinute;
///
///   @override
///   Middleware toMiddleware() => ...;
/// }
/// ```
abstract interface class Layer {
  /// Builds the shelf middleware this value configures.
  Middleware toMiddleware();
}

/// Turns a mixed list of [Layer] and shelf [Middleware] values into
/// shelf middleware, keeping the order they were written in.
///
/// Plain shelf middleware is allowed at the composition site for anything that
/// never has to fit in an annotation.
List<Middleware> resolveMiddleware(List<Object> entries) {
  return [
    for (final entry in entries)
      if (entry is Layer)
        entry.toMiddleware()
      else if (entry is Middleware)
        entry
      else
        throw ArgumentError(
          'a layer must be a Layer or a shelf Middleware, '
          'got ${entry.runtimeType}',
        ),
  ];
}
