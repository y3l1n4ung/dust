import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../response/rejection.dart';
import 'extractable.dart';

/// The request-context key state of type [T] travels under.
///
/// The key is the type's name, which is all Dart exposes without mirrors. Two
/// classes with the same name from different libraries therefore share a key,
/// and the second `withState` wins. [StateExtractable] catches that: the value
/// it finds fails its type test and answers 500 rather than handing a handler
/// the wrong object. Renaming one of them is the fix.
String stateKeyFor<T>() => 'dust_server/state/$T';

/// Reads application state attached with `Router.withState`.
///
/// `@State() NoteRepo repo` lowers to this. It is the counterpart to axum's
/// `State<S>`: the parameter's type selects the value, so nothing has to name a
/// key on both sides.
///
/// Missing state is a 500. The handler cannot run, and the fault is a
/// composition mistake rather than anything the client did.
final class StateExtractable<T extends Object> implements FromRequestParts<T> {
  /// Reads the state of type [T].
  const StateExtractable();

  @override
  Future<Result<T, Rejection>> extract(Request request) async {
    final value = request.context[stateKeyFor<T>()];
    if (value == null) {
      return Err(
        Rejection.internal(
          'no $T state is attached; add withState<$T>(...) where the routes '
          'are mounted',
        ),
      );
    }
    if (value is! T) {
      return Err(
          Rejection.internal('the attached $T state has the wrong type'));
    }
    return Ok(value);
  }
}

/// Builds the middleware that puts [state] on every request below a group.
///
/// Returns `null` when a group attached nothing, so groups without state add
/// no layer at all.
Middleware? stateMiddleware(Map<String, Object> state) {
  if (state.isEmpty) return null;
  final snapshot = Map<String, Object>.unmodifiable(state);
  return (Handler inner) {
    return (Request request) => inner(request.change(context: snapshot));
  };
}
