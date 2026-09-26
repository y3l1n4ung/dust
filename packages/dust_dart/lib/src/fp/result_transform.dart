import 'result.dart';

/// Transforming and choosing between [Result] values.
///
/// ```dart
/// final label = const Ok<int, String>(3).mapOr('failed', (n) => '$n items');
/// final chosen = const Err<int, String>('cache').or(const Ok<int, String>(1));
/// ```
extension ResultTransform<T, E> on Result<T, E> {
  /// Maps a successful value, or returns [fallback] for an [Err].
  ///
  /// [fallback] is evaluated before the call. Use [mapOrElse] when computing it
  /// is expensive or needs the error.
  ///
  /// ```dart
  /// final length = const Ok<String, int>('John').mapOr(0, (name) => name.length);
  /// ```
  R mapOr<R>(R fallback, R Function(T value) mapper) {
    return switch (this) {
      Ok<T, E>(:final value) => mapper(value),
      Err<T, E>() => fallback,
    };
  }

  /// Maps a successful value, or computes a fallback from the error.
  ///
  /// Only the branch that applies is called.
  ///
  /// ```dart
  /// final label = const Err<int, String>('offline')
  ///     .mapOrElse((error) => 'failed: $error', (n) => '$n items');
  /// ```
  R mapOrElse<R>(R Function(E error) fallback, R Function(T value) mapper) {
    return switch (this) {
      Ok<T, E>(:final value) => mapper(value),
      Err<T, E>(:final error) => fallback(error),
    };
  }

  /// Calls [action] with a successful value, and returns this result unchanged.
  ///
  /// ```dart
  /// final saved = const Ok<int, String>(7).inspect((id) => print('saved $id'));
  /// ```
  Result<T, E> inspect(void Function(T value) action) {
    if (this case Ok<T, E>(:final value)) action(value);
    return this;
  }

  /// Calls [action] with the error, and returns this result unchanged.
  ///
  /// ```dart
  /// final saved = const Err<int, String>('locked')
  ///     .inspectErr((error) => print('save failed: $error'));
  /// ```
  Result<T, E> inspectErr(void Function(E error) action) {
    if (this case Err<T, E>(:final error)) action(error);
    return this;
  }

  /// Returns [other] when this result succeeded, or this error otherwise.
  ///
  /// [other] is evaluated before the call. Use `andThen` when the next result
  /// needs the value or is expensive to build.
  ///
  /// ```dart
  /// final next = const Ok<int, String>(1).and(const Ok<String, String>('a'));
  /// ```
  Result<R, E> and<R>(Result<R, E> other) {
    return switch (this) {
      Ok<T, E>() => other,
      Err<T, E>(:final error) => Err<R, E>(error),
    };
  }

  /// Returns this result when it succeeded, or [other] otherwise.
  ///
  /// [other] is evaluated before the call. Use `orElse` when the fallback
  /// needs the error or is expensive to build.
  ///
  /// ```dart
  /// final value = const Err<int, String>('cache miss')
  ///     .or(const Ok<int, String>(42)); // Ok(42)
  /// ```
  Result<T, F> or<F>(Result<T, F> other) {
    return switch (this) {
      Ok<T, E>(:final value) => Ok<T, F>(value),
      Err<T, E>() => other,
    };
  }
}
