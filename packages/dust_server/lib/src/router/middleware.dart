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

/// A [Layer] that owns something and needs telling when the server stops.
///
/// A rate limiter holding a client, a metrics layer with a flusher, an auth
/// layer caching keys on a timer: each acquires a resource when it is built and
/// has nowhere to release it, because [Layer] declares only [Layer.toMiddleware].
/// axum needs no equivalent — Rust drops a layer when its router goes — so this
/// is Dart's problem rather than a borrowed shape.
///
/// `close(drain:)` disposes every one it finds on the router, inside the same
/// budget it gives requests and background work.
///
/// It is a separate interface because Dart's `implements` requires every member
/// to be re-declared, so adding a method to [Layer] — even one with a body —
/// would break every existing layer.
///
/// ```dart
/// final class RateLimit implements DisposableLayer {
///   RateLimit(this._client);
///
///   final RedisClient _client;
///
///   @override
///   Middleware toMiddleware() => ...;
///
///   @override
///   Future<void> dispose() => _client.close();
/// }
/// ```
abstract interface class DisposableLayer implements Layer {
  /// Releases whatever this layer acquired.
  ///
  /// Called once, during shutdown. It should not throw; one that does is
  /// reported through the router's `onError` and does not stop the others.
  Future<void> dispose();
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
