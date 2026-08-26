import 'dart:async';

import 'package:shelf/shelf.dart';

/// Something that answers a request.
///
/// The Dart counterpart of tower's `Service`, which axum's `serve` is generic
/// over. Its `call` method is the whole contract, so an implementation is
/// assignable to a shelf [Handler] without conversion: Dart tears off `call`
/// implicitly when an object is used where a function type is wanted.
///
/// tower's `poll_ready` has no counterpart and cannot have one — a function
/// cannot decline before it is called — so backpressure is not expressible
/// here. See `docs/design/server-axum-parity.md`.
abstract interface class Service {
  /// Answers [request].
  FutureOr<Response> call(Request request);
}
