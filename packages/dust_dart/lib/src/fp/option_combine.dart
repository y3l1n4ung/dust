import 'option.dart';
import 'result.dart';

/// Combining [Option] values, and moving between [Option] and [Result].
///
/// ```dart
/// final both = const Some<int>(1).zip(const Some<String>('a')); // Some((1, a))
/// final checked = const None<int>().okOr('missing'); // Err(missing)
/// ```
extension OptionCombine<T> on Option<T> {
  /// Pairs this value with [other]'s, when both are present.
  ///
  /// ```dart
  /// final point = const Some<int>(1).zip(const Some<int>(2)); // Some((1, 2))
  /// ```
  Option<(T, U)> zip<U>(Option<U> other) {
    return switch ((this, other)) {
      (Some<T>(value: final left), Some<U>(value: final right)) =>
        Some<(T, U)>((left, right)),
      _ => None<(T, U)>(),
    };
  }

  /// Combines this value with [other]'s, when both are present.
  ///
  /// [combine] is not called unless both are present.
  ///
  /// ```dart
  /// final sum = const Some<int>(1).zipWith(const Some<int>(2), (a, b) => a + b);
  /// ```
  Option<R> zipWith<U, R>(
      Option<U> other, R Function(T left, U right) combine) {
    return switch ((this, other)) {
      (Some<T>(value: final left), Some<U>(value: final right)) =>
        Some<R>(combine(left, right)),
      _ => None<R>(),
    };
  }

  /// Returns [Ok] with a present value, or [Err] with [error] when absent.
  ///
  /// A present `null` stays present: `Some<T?>(null)` becomes `Ok(null)`, not
  /// an error, because the value was there.
  ///
  /// ```dart
  /// final id = const None<int>().okOr('no id'); // Err(no id)
  /// ```
  Result<T, E> okOr<E>(E error) {
    return switch (this) {
      Some<T>(:final value) => Ok<T, E>(value),
      None<T>() => Err<T, E>(error),
    };
  }

  /// Returns [Ok] with a present value, or computes an [Err] when absent.
  ///
  /// [error] is not called when the option is present.
  ///
  /// ```dart
  /// final id = const None<int>().okOrElse(() => 'no id'); // Err(no id)
  /// ```
  Result<T, E> okOrElse<E>(E Function() error) {
    return switch (this) {
      Some<T>(:final value) => Ok<T, E>(value),
      None<T>() => Err<T, E>(error()),
    };
  }
}

/// Splitting an [Option] of a pair.
extension OptionUnzip<A, B> on Option<(A, B)> {
  /// Splits a present pair into two present options, and absence into two
  /// absent ones.
  ///
  /// ```dart
  /// final (left, right) = const Some<(int, String)>((1, 'a')).unzip();
  /// ```
  (Option<A>, Option<B>) unzip() {
    return switch (this) {
      Some<(A, B)>(value: (final left, final right)) => (
          Some<A>(left),
          Some<B>(right),
        ),
      None<(A, B)>() => (None<A>(), None<B>()),
    };
  }
}

/// Removing one level of [Option] nesting.
extension OptionFlatten<T> on Option<Option<T>> {
  /// Turns `Some(Some(value))` into `Some(value)`, and any absence into
  /// [None].
  ///
  /// ```dart
  /// final flat = const Some<Option<int>>(Some<int>(1)).flatten(); // Some(1)
  /// ```
  Option<T> flatten() {
    return switch (this) {
      Some<Option<T>>(:final value) => value,
      None<Option<T>>() => None<T>(),
    };
  }
}

/// Swapping an [Option] of a [Result] inside out.
extension OptionTranspose<T, E> on Option<Result<T, E>> {
  /// Turns an optional result into a result of an option.
  ///
  /// `None()` becomes `Ok(None())`, `Some(Ok(v))` becomes `Ok(Some(v))`, and
  /// `Some(Err(e))` becomes `Err(e)` — so a failure is never hidden inside a
  /// present option.
  ///
  /// ```dart
  /// const Option<Result<int, String>> parsed = Some(Ok(1));
  /// final swapped = parsed.transpose(); // Ok(Some(1))
  /// ```
  Result<Option<T>, E> transpose() {
    return switch (this) {
      None<Result<T, E>>() => Ok<Option<T>, E>(None<T>()),
      Some<Result<T, E>>(value: Ok<T, E>(:final value)) =>
        Ok<Option<T>, E>(Some<T>(value)),
      Some<Result<T, E>>(value: Err<T, E>(:final error)) =>
        Err<Option<T>, E>(error),
    };
  }
}
