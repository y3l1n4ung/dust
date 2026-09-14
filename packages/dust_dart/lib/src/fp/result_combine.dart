import 'option.dart';
import 'result.dart';

/// Removing one level of [Result] nesting.
extension ResultFlatten<T, E> on Result<Result<T, E>, E> {
  /// Turns `Ok(Ok(value))` into `Ok(value)`, and either error into [Err].
  ///
  /// ```dart
  /// const Result<Result<int, String>, String> nested = Ok(Ok(1));
  /// final flat = nested.flatten(); // Ok(1)
  /// ```
  Result<T, E> flatten() {
    return switch (this) {
      Ok<Result<T, E>, E>(:final value) => value,
      Err<Result<T, E>, E>(:final error) => Err<T, E>(error),
    };
  }
}

/// Swapping a [Result] of an [Option] inside out.
extension ResultTranspose<T, E> on Result<Option<T>, E> {
  /// Turns a result of an option into an optional result.
  ///
  /// `Ok(None())` becomes `None()`, `Ok(Some(v))` becomes `Some(Ok(v))`, and
  /// `Err(e)` becomes `Some(Err(e))`, so an error is never mistaken for
  /// absence. It is the inverse of `Option.transpose`.
  ///
  /// ```dart
  /// const Result<Option<int>, String> row = Ok(Some(1));
  /// final swapped = row.transpose(); // Some(Ok(1))
  /// ```
  Option<Result<T, E>> transpose() {
    return switch (this) {
      Ok<Option<T>, E>(value: None<T>()) => None<Result<T, E>>(),
      Ok<Option<T>, E>(value: Some<T>(:final value)) =>
        Some<Result<T, E>>(Ok<T, E>(value)),
      Err<Option<T>, E>(:final error) => Some<Result<T, E>>(Err<T, E>(error)),
    };
  }
}

/// Gathering many [Result] values into one.
extension ResultCollect<T, E> on Iterable<Result<T, E>> {
  /// Returns every successful value in order, or the first error.
  ///
  /// Stops at the first [Err]: later elements of a lazy iterable are not read.
  ///
  /// ```dart
  /// final ids = [const Ok<int, String>(1), const Ok<int, String>(2)].collect();
  /// // Ok([1, 2])
  /// ```
  Result<List<T>, E> collect() {
    final values = <T>[];
    for (final result in this) {
      switch (result) {
        case Ok<T, E>(:final value):
          values.add(value);
        case Err<T, E>(:final error):
          return Err<List<T>, E>(error);
      }
    }
    return Ok<List<T>, E>(values);
  }
}
