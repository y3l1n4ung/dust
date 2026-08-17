import 'package:dust_dart/serde.dart';
import 'package:shelf/shelf.dart';

import 'encoders.dart';
import 'into_response.dart';
import 'rejection.dart';

/// Turns whatever a typed handler returned into a response.
///
/// The rule is the one axum uses for `IntoResponse`: the handler says what it
/// produced, and the runtime decides how it reaches the wire. That is what
/// lets a handler end in `repository.find(id)` rather than in a call to an
/// encoder.
///
/// | Returned | Answer |
/// | :--- | :--- |
/// | `Response` | itself, untouched |
/// | `IntoResponse` | `intoResponse()`, which covers `Rejection` |
/// | `Ok(value)` | the value, dispatched again |
/// | `Err(error)` | the error, dispatched again, defaulting to 500 |
/// | `Some(value)` | the value, dispatched again |
/// | `None` | 404 |
/// | `null` | 204 |
/// | anything else | JSON, through `serialize` when it derives `Serialize` |
///
/// [status] applies only to the last row and to `Ok`, so a create can answer
/// 201 without the failure path inheriting it.
Response responseFrom(Object? value, {int status = 200}) {
  return switch (value) {
    null => noContent(),
    final Response response => response,
    final IntoResponse into => into.intoResponse(),
    Ok(value: final inner) => responseFrom(inner, status: status),
    Err(error: final inner) => _errorResponse(inner),
    Some(value: final inner) => responseFrom(inner, status: status),
    None() => const Rejection.notFound('not found').intoResponse(),
    _ => jsonResponse(value, status: status),
  };
}

/// Answers for the error side of a [Result].
///
/// A handler that returns `Err` has said the request failed, so an error with
/// no opinion about status becomes a 500 rather than a 200 carrying an error
/// object. Anything implementing [IntoResponse] keeps its own status.
Response _errorResponse(Object? error) {
  return switch (error) {
    null => const Rejection.internal().intoResponse(),
    final Response response => response,
    final IntoResponse into => into.intoResponse(),
    final Serializable body => jsonResponse(body, status: 500),
    _ => Rejection.internal(error.toString()).intoResponse(),
  };
}
