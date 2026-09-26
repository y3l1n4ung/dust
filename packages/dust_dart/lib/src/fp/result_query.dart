import 'option.dart';
import 'result.dart';

/// Asking a [Result] what it holds, and taking the value out.
///
/// ```dart
/// final passed = const Ok<int, String>(90).isOkAnd((score) => score >= 50);
/// final score = const Ok<int, String>(90).ok(); // Some(90)
/// ```
extension ResultQuery<T, E> on Result<T, E> {
  /// Whether the result succeeded and its value satisfies [predicate].
  ///
  /// [predicate] is not called for an [Err].
  ///
  /// ```dart
  /// final passed = const Ok<int, String>(90).isOkAnd((score) => score >= 50);
  /// ```
  bool isOkAnd(bool Function(T value) predicate) {
    return switch (this) {
      Ok<T, E>(:final value) => predicate(value),
      Err<T, E>() => false,
    };
  }

  /// Whether the result failed and its error satisfies [predicate].
  ///
  /// [predicate] is not called for an [Ok].
  ///
  /// ```dart
  /// final retry = const Err<int, int>(503).isErrAnd((status) => status >= 500);
  /// ```
  bool isErrAnd(bool Function(E error) predicate) {
    return switch (this) {
      Ok<T, E>() => false,
      Err<T, E>(:final error) => predicate(error),
    };
  }

  /// The successful value as [Some], or [None] for an [Err].
  ///
  /// The error is dropped. Use it where failure only means "no value".
  ///
  /// ```dart
  /// final score = const Err<int, String>('offline').ok(); // None()
  /// ```
  Option<T> ok() {
    return switch (this) {
      Ok<T, E>(:final value) => Some<T>(value),
      Err<T, E>() => None<T>(),
    };
  }

  /// The error as [Some], or [None] for an [Ok].
  ///
  /// ```dart
  /// final reason = const Err<int, String>('offline').err(); // Some(offline)
  /// ```
  Option<E> err() {
    return switch (this) {
      Ok<T, E>() => None<E>(),
      Err<T, E>(:final error) => Some<E>(error),
    };
  }

  /// Returns the successful value, or throws for an [Err].
  ///
  /// Throws a [StateError] naming the error. Prefer [expect] where the reason
  /// the operation cannot fail is worth saying, and a `switch` or `unwrapOr`
  /// wherever it can.
  ///
  /// ```dart
  /// final id = const Ok<int, String>(7).unwrap(); // 7
  /// ```
  T unwrap() {
    return switch (this) {
      Ok<T, E>(:final value) => value,
      Err<T, E>(:final error) =>
        throw StateError('called unwrap() on Err: $error'),
    };
  }

  /// Returns the successful value, or throws [message] for an [Err].
  ///
  /// Throws a [StateError] carrying [message] followed by the error, so the
  /// failure says both why success was expected and what went wrong.
  ///
  /// ```dart
  /// final port = const Ok<int, String>(8080).expect('the config has a port');
  /// ```
  T expect(String message) {
    return switch (this) {
      Ok<T, E>(:final value) => value,
      Err<T, E>(:final error) => throw StateError('$message: $error'),
    };
  }

  /// Returns the error, or throws for an [Ok].
  ///
  /// Throws a [StateError] naming the value. Mostly useful in tests.
  ///
  /// ```dart
  /// final reason = const Err<int, String>('offline').unwrapErr(); // offline
  /// ```
  E unwrapErr() {
    return switch (this) {
      Ok<T, E>(:final value) =>
        throw StateError('called unwrapErr() on Ok: $value'),
      Err<T, E>(:final error) => error,
    };
  }

  /// Returns the error, or throws [message] for an [Ok].
  ///
  /// Throws a [StateError] carrying [message] followed by the value.
  ///
  /// ```dart
  /// final reason = const Err<int, String>('offline').expectErr('no network');
  /// ```
  E expectErr(String message) {
    return switch (this) {
      Ok<T, E>(:final value) => throw StateError('$message: $value'),
      Err<T, E>(:final error) => error,
    };
  }
}
