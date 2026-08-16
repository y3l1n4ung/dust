import 'package:shelf/shelf.dart';

import 'route.dart';

/// The handlers registered for one path, keyed by method.
///
/// Built with the verb functions in this library and chained the way axum
/// chains `get(list).post(create)`:
///
/// ```dart
/// final notes = Router.prefixed('/notes')
///   ..route('/', get(listNotes).post(writeNote))
///   ..route('/{id}', get(readNote).delete(removeNote));
/// ```
final class MethodRouter {
  /// Starts an empty set.
  const MethodRouter() : this._(const {});

  const MethodRouter._(this.handlers);

  /// The handlers registered so far, keyed by uppercase method.
  final Map<String, Handler> handlers;

  /// Returns a set with [handler] added for [method].
  ///
  /// Each step returns a new value rather than mutating, so a half-built chain
  /// cannot be aliased into two routes.
  MethodRouter on(String method, Handler handler) {
    final verb =
        method == Route.anyMethod ? Route.anyMethod : method.toUpperCase();
    if (handlers.containsKey(verb)) {
      throw ArgumentError('$verb is already registered for this path');
    }
    return MethodRouter._(Map.unmodifiable({...handlers, verb: handler}));
  }

  /// Adds a `GET` handler.
  MethodRouter get(Handler handler) => on('GET', handler);

  /// Adds a `POST` handler.
  MethodRouter post(Handler handler) => on('POST', handler);

  /// Adds a `PUT` handler.
  MethodRouter put(Handler handler) => on('PUT', handler);

  /// Adds a `PATCH` handler.
  MethodRouter patch(Handler handler) => on('PATCH', handler);

  /// Adds a `DELETE` handler.
  MethodRouter delete(Handler handler) => on('DELETE', handler);

  /// Adds a `HEAD` handler.
  MethodRouter head(Handler handler) => on('HEAD', handler);

  /// Adds an `OPTIONS` handler.
  MethodRouter options(Handler handler) => on('OPTIONS', handler);

  /// Adds a handler for every method no other handler claims.
  ///
  /// The counterpart to axum's `any` and `shelf_router`'s `all`. A method with
  /// its own handler still wins, so `get(read).any(rest)` sends `GET` to
  /// `read` and everything else to `rest`.
  MethodRouter any(Handler handler) => on(Route.anyMethod, handler);
}

/// Serves [handler] for `GET`.
MethodRouter get(Handler handler) => MethodRouter().get(handler);

/// Serves [handler] for `POST`.
MethodRouter post(Handler handler) => MethodRouter().post(handler);

/// Serves [handler] for `PUT`.
MethodRouter put(Handler handler) => MethodRouter().put(handler);

/// Serves [handler] for `PATCH`.
MethodRouter patch(Handler handler) => MethodRouter().patch(handler);

/// Serves [handler] for `DELETE`.
MethodRouter delete(Handler handler) => MethodRouter().delete(handler);

/// Serves [handler] for `HEAD`.
MethodRouter head(Handler handler) => MethodRouter().head(handler);

/// Serves [handler] for `OPTIONS`.
MethodRouter options(Handler handler) => MethodRouter().options(handler);

/// Serves [handler] for any method no other handler claims.
MethodRouter any(Handler handler) => MethodRouter().any(handler);
