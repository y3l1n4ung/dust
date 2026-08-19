import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../response/rejection.dart';
import 'extractable.dart';

/// Runs an extractor and throws the rejection instead of returning it.
///
/// `extract` hands back a `Result` because generated code decides what to do
/// with each failure. Hand-written handlers almost always want the same thing,
/// which is to stop and answer with the rejection, and threading a `Result`
/// through every line to say so buries the handler in casts.
///
/// The throw is caught by `guard`, so a handler wrapped in one reads top to
/// bottom:
///
/// ```dart
/// Future<Response> readTodo(Request request) => guard(() async {
///       final id = await const PathExtractable<String>('id').require(request);
///       final repo = await const StateExtractable<TodoRepo>().require(request);
///       return jsonResponse(repo.find(id).toJson());
///     });
/// ```
///
/// Outside a `guard` the rejection escapes as an exception, which is the same
/// mistake as letting any other exception escape a handler.
extension RequireExtraction<T> on FromRequestParts<T> {
  /// Extracts the value, throwing the [Rejection] when extraction fails.
  Future<T> require(Request request) async => (await extract(request)).orReject;
}

/// Collapses an extraction outcome onto its value.
extension RejectOnFailure<T> on Result<T, Rejection> {
  /// The extracted value, or a thrown [Rejection].
  T get orReject => switch (this) {
        Ok(:final value) => value,
        Err(:final error) => throw error,
      };
}
