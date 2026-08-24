import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../extraction/extractable.dart';
import '../response/rejection.dart';

/// The context key a value extracted by [fromExtractor] travels under.
///
/// Keyed by type name, the same way state is, so two extensions of different
/// types coexist without knowing about each other.
String extensionKeyFor<T>() => 'dust_server/extension/$T';

/// Runs [extractor] as middleware, and passes its value on to the handler.
///
/// The counterpart to axum's `middleware::from_extractor`. A layer can pass a
/// request on or answer it, so an extractor used as one needs somewhere to put
/// what it produced: it goes in the request context, and [Extension] reads it
/// back.
///
/// ```dart
/// final admin = Router()
///   ..routeLayer(fromExtractor(const RequireScope('admin')))
///   ..route('/orders', get(listOrders));
/// ```
///
/// Two things this buys over naming the extractor in every handler. The work
/// happens **once** for a request rather than once per handler that wants the
/// value. And a route added later cannot forget it, which turns a code review
/// into a compile-time arrangement.
///
/// Pair it with `routeLayer` rather than `layer` for a guard: a path that does
/// not exist should answer 404, not 401.
Middleware fromExtractor<T extends Object>(FromRequestParts<T> extractor) {
  return (Handler inner) {
    return (Request request) async {
      switch (await extractor.extract(request)) {
        case Err(:final error):
          return error.intoResponse();
        case Ok(:final value):
          return inner(
            request.change(context: {extensionKeyFor<T>(): value}),
          );
      }
    };
  };
}

/// Reads a value a [fromExtractor] layer already produced.
///
/// The counterpart to axum's `Extension<T>`.
///
/// A missing value is a **500**, not a 401: the layer not being installed is a
/// wiring mistake in the route table rather than something a client did, and
/// answering 401 would send whoever is debugging it after a credential that was
/// never the problem. The message says which type is missing.
final class Extension<T extends Object> implements FromRequestParts<T> {
  /// Reads the [T] a `fromExtractor` layer left behind.
  const Extension();

  @override
  Future<Result<T, Rejection>> extract(Request request) async {
    final carried = request.context[extensionKeyFor<T>()];
    if (carried is T) return Ok(carried);

    return Err(
      Rejection.internal(
        'no $T in the request; add fromExtractor(...) where these routes are '
        'mounted',
      ),
    );
  }
}
