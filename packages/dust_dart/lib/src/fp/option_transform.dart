import 'option.dart';

/// Transforming and choosing between [Option] values.
///
/// ```dart
/// final label = const Some<int>(3).mapOr('none', (count) => '$count items');
/// final chosen = const None<int>().or(const Some<int>(1)); // Some(1)
/// ```
extension OptionTransform<T> on Option<T> {
  /// Maps a present value, or returns [fallback] when absent.
  ///
  /// [fallback] is evaluated before the call. Use [mapOrElse] when computing it
  /// is expensive or has side effects.
  ///
  /// ```dart
  /// final length = const Some<String>('John').mapOr(0, (name) => name.length);
  /// ```
  R mapOr<R>(R fallback, R Function(T value) mapper) {
    return switch (this) {
      Some<T>(:final value) => mapper(value),
      None<T>() => fallback,
    };
  }

  /// Maps a present value, or computes a fallback when absent.
  ///
  /// Only the branch that applies is called.
  ///
  /// ```dart
  /// final length = const None<String>().mapOrElse(() => 0, (name) => name.length);
  /// ```
  R mapOrElse<R>(R Function() fallback, R Function(T value) mapper) {
    return switch (this) {
      Some<T>(:final value) => mapper(value),
      None<T>() => fallback(),
    };
  }

  /// Calls [action] with a present value, and returns this option unchanged.
  ///
  /// ```dart
  /// final id = const Some<int>(7).inspect(print); // prints 7, returns Some(7)
  /// ```
  Option<T> inspect(void Function(T value) action) {
    if (this case Some<T>(:final value)) {
      action(value);
    }
    return this;
  }

  /// Keeps a present value only when it satisfies [predicate].
  ///
  /// ```dart
  /// final even = const Some<int>(3).filter((value) => value.isEven); // None()
  /// ```
  Option<T> filter(bool Function(T value) predicate) {
    return switch (this) {
      Some<T>(:final value) when predicate(value) => this,
      _ => None<T>(),
    };
  }

  /// Returns [other] when this option is present, and [None] otherwise.
  ///
  /// ```dart
  /// final next = const Some<int>(1).and(const Some<String>('a')); // Some(a)
  /// ```
  Option<R> and<R>(Option<R> other) {
    return switch (this) {
      Some<T>() => other,
      None<T>() => None<R>(),
    };
  }

  /// Returns this option when present, and [other] otherwise.
  ///
  /// [other] is evaluated before the call. Use [orElse] when computing it is
  /// expensive or has side effects.
  ///
  /// ```dart
  /// final name = const None<String>().or(const Some<String>('guest'));
  /// ```
  Option<T> or(Option<T> other) {
    return switch (this) {
      Some<T>() => this,
      None<T>() => other,
    };
  }

  /// Returns this option when present, and computes one otherwise.
  ///
  /// [fallback] is not called when this option is present.
  ///
  /// ```dart
  /// final name = const None<String>().orElse(() => const Some<String>('guest'));
  /// ```
  Option<T> orElse(Option<T> Function() fallback) {
    return switch (this) {
      Some<T>() => this,
      None<T>() => fallback(),
    };
  }

  /// Returns whichever of the two options is present, when exactly one is.
  ///
  /// ```dart
  /// final one = const Some<int>(1).xor(const None<int>()); // Some(1)
  /// final both = const Some<int>(1).xor(const Some<int>(2)); // None()
  /// ```
  Option<T> xor(Option<T> other) {
    return switch ((this, other)) {
      (Some<T>(), None<T>()) => this,
      (None<T>(), Some<T>()) => other,
      _ => None<T>(),
    };
  }
}
