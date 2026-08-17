import 'dart:async';

import 'package:shelf/shelf.dart';

import '../response/dispatch.dart';
import '../response/encoders.dart';
import 'route.dart';

/// What a verb builder accepts: a function of the request that returns
/// whatever the endpoint produced.
///
/// A plain `shelf` [Handler] is one of these, since a `Response` is a value
/// like any other and `responseFrom` passes it through untouched. That is what
/// lets a route take either without a second set of builders.
typedef Endpoint<T> = FutureOr<T> Function(Request request);

/// The handlers registered for one path, keyed by method.
///
/// Built with the verb functions in this library and chained the way axum
/// chains `get(list).post(create)`:
///
/// ```dart
/// final notes = Router.prefixed('/notes')
///   ..route('/', get(listNotes).post(writeNote, status: 201))
///   ..route('/{id}', get(readNote).delete(removeNote));
/// ```
///
/// An endpoint returns a value and the verb builder turns it into a response:
/// a model becomes JSON through its `serialize`, `null` becomes 204, a
/// `Rejection` becomes its own status, and anything thrown that is not a
/// rejection becomes an opaque 500 with the real error going to the router's
/// `onError`. Nothing in an endpoint builds a response or catches an error.
final class MethodRouter {
  /// Starts an empty set.
  const MethodRouter() : this._(const {});

  const MethodRouter._(this.handlers);

  /// The handlers registered so far, keyed by uppercase method.
  final Map<String, Handler> handlers;

  /// Returns a set with [endpoint] added for [method].
  ///
  /// Each step returns a new value rather than mutating, so a half-built chain
  /// cannot be aliased into two routes. [status] is the success status, which
  /// is how a create says 201; a failure keeps its own.
  MethodRouter on<T>(String method, Endpoint<T> endpoint, {int status = 200}) {
    final verb =
        method == Route.anyMethod ? Route.anyMethod : method.toUpperCase();
    if (handlers.containsKey(verb)) {
      throw ArgumentError('$verb is already registered for this path');
    }
    return MethodRouter._(
      Map.unmodifiable({...handlers, verb: _adapt(endpoint, status)}),
    );
  }

  /// Adds a `GET` endpoint.
  MethodRouter get<T>(Endpoint<T> endpoint, {int status = 200}) =>
      on('GET', endpoint, status: status);

  /// Adds a `POST` endpoint.
  MethodRouter post<T>(Endpoint<T> endpoint, {int status = 200}) =>
      on('POST', endpoint, status: status);

  /// Adds a `PUT` endpoint.
  MethodRouter put<T>(Endpoint<T> endpoint, {int status = 200}) =>
      on('PUT', endpoint, status: status);

  /// Adds a `PATCH` endpoint.
  MethodRouter patch<T>(Endpoint<T> endpoint, {int status = 200}) =>
      on('PATCH', endpoint, status: status);

  /// Adds a `DELETE` endpoint.
  MethodRouter delete<T>(Endpoint<T> endpoint, {int status = 200}) =>
      on('DELETE', endpoint, status: status);

  /// Adds a `HEAD` endpoint.
  MethodRouter head<T>(Endpoint<T> endpoint, {int status = 200}) =>
      on('HEAD', endpoint, status: status);

  /// Adds an `OPTIONS` endpoint.
  MethodRouter options<T>(Endpoint<T> endpoint, {int status = 200}) =>
      on('OPTIONS', endpoint, status: status);

  /// Adds a `TRACE` endpoint.
  ///
  /// Rarely wanted: the specification has it echo the request back, which
  /// leaks headers a proxy added, so most deployments refuse it outright. It
  /// is here so a router can answer rather than 405 when one is needed.
  MethodRouter trace<T>(Endpoint<T> endpoint, {int status = 200}) =>
      on('TRACE', endpoint, status: status);

  /// Adds a `CONNECT` endpoint.
  MethodRouter connect<T>(Endpoint<T> endpoint, {int status = 200}) =>
      on('CONNECT', endpoint, status: status);

  /// Adds an endpoint for every method no other handler claims.
  ///
  /// The counterpart to axum's `any` and `shelf_router`'s `all`. A method with
  /// its own handler still wins, so `get(read).any(rest)` sends `GET` to
  /// `read` and everything else to `rest`.
  MethodRouter any<T>(Endpoint<T> endpoint, {int status = 200}) =>
      on(Route.anyMethod, endpoint, status: status);
}

/// Wraps [endpoint] so it answers with a response however it finished.
Handler _adapt<T>(Endpoint<T> endpoint, int status) {
  return (request) =>
      guard(() async => responseFrom(await endpoint(request), status: status));
}

/// Serves [endpoint] for `GET`.
MethodRouter get<T>(Endpoint<T> endpoint, {int status = 200}) =>
    const MethodRouter().get(endpoint, status: status);

/// Serves [endpoint] for `POST`.
MethodRouter post<T>(Endpoint<T> endpoint, {int status = 200}) =>
    const MethodRouter().post(endpoint, status: status);

/// Serves [endpoint] for `PUT`.
MethodRouter put<T>(Endpoint<T> endpoint, {int status = 200}) =>
    const MethodRouter().put(endpoint, status: status);

/// Serves [endpoint] for `PATCH`.
MethodRouter patch<T>(Endpoint<T> endpoint, {int status = 200}) =>
    const MethodRouter().patch(endpoint, status: status);

/// Serves [endpoint] for `DELETE`.
MethodRouter delete<T>(Endpoint<T> endpoint, {int status = 200}) =>
    const MethodRouter().delete(endpoint, status: status);

/// Serves [endpoint] for `HEAD`.
MethodRouter head<T>(Endpoint<T> endpoint, {int status = 200}) =>
    const MethodRouter().head(endpoint, status: status);

/// Serves [endpoint] for `OPTIONS`.
MethodRouter options<T>(Endpoint<T> endpoint, {int status = 200}) =>
    const MethodRouter().options(endpoint, status: status);

/// Serves [endpoint] for any method no other handler claims.
MethodRouter any<T>(Endpoint<T> endpoint, {int status = 200}) =>
    const MethodRouter().any(endpoint, status: status);

/// Serves [endpoint] for `TRACE`.
MethodRouter trace<T>(Endpoint<T> endpoint, {int status = 200}) =>
    const MethodRouter().trace(endpoint, status: status);

/// Serves [endpoint] for `CONNECT`.
MethodRouter connect<T>(Endpoint<T> endpoint, {int status = 200}) =>
    const MethodRouter().connect(endpoint, status: status);
