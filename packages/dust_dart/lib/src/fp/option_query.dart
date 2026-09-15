import 'option.dart';

/// Reading an [Option] without matching on it.
///
/// ```dart
/// const nickname = Some<String>('John');
/// final hasJohn = nickname.contains('John'); // true
/// final text = nickname.expect('a nickname was set'); // 'John'
/// ```
extension OptionQuery<T> on Option<T> {
  /// Returns the present value, or `null` when absent.
  ///
  /// This collapses `None()` and `Some<T?>(null)` into the same `null`, which
  /// is exactly the distinction [Option] exists to keep. Use it at the edge
  /// where code wants a plain nullable value back.
  ///
  /// ```dart
  /// final value = const Some<int>(1).toNullable(); // 1
  /// final missing = const None<int>().toNullable(); // null
  /// ```
  T? toNullable() {
    return switch (this) {
      Some<T>(:final value) => value,
      None<T>() => null,
    };
  }

  /// Returns the present value as a one-element iterable, or an empty one.
  ///
  /// ```dart
  /// final names = [
  ///   ...const Some<String>('John').toIterable(),
  ///   ...const None<String>().toIterable(),
  /// ]; // ['John']
  /// ```
  Iterable<T> toIterable() {
    return switch (this) {
      Some<T>(:final value) => [value],
      None<T>() => const [],
    };
  }

  /// Whether this option holds a value equal to [value].
  ///
  /// ```dart
  /// final isAdmin = const Some<String>('admin').contains('admin'); // true
  /// ```
  bool contains(T value) {
    return switch (this) {
      Some<T>(value: final present) => present == value,
      None<T>() => false,
    };
  }

  /// Whether a value is present and satisfies [predicate].
  ///
  /// [predicate] is not called when the option is absent.
  ///
  /// ```dart
  /// final isAdult = const Some<int>(21).isSomeAnd((age) => age >= 18); // true
  /// ```
  bool isSomeAnd(bool Function(T value) predicate) {
    return switch (this) {
      Some<T>(:final value) => predicate(value),
      None<T>() => false,
    };
  }

  /// Whether the option is absent, or its value satisfies [predicate].
  ///
  /// [predicate] is not called when the option is absent.
  ///
  /// ```dart
  /// final allowed = const None<int>().isNoneOr((age) => age >= 18); // true
  /// ```
  bool isNoneOr(bool Function(T value) predicate) {
    return switch (this) {
      Some<T>(:final value) => predicate(value),
      None<T>() => true,
    };
  }

  /// Returns the present value, or throws when absent.
  ///
  /// Throws a [StateError] for [None]. Prefer [expect] where the reason a
  /// value must exist is worth saying, and [OptionUnwrap.unwrapOr] where it need
  /// not exist at all.
  ///
  /// ```dart
  /// final id = const Some<int>(7).unwrap(); // 7
  /// ```
  T unwrap() {
    return switch (this) {
      Some<T>(:final value) => value,
      None<T>() => throw StateError('called unwrap() on None'),
    };
  }

  /// Returns the present value, or throws [message] when absent.
  ///
  /// Throws a [StateError] carrying [message] for [None], so the failure says
  /// why the value was expected rather than only that it was missing.
  ///
  /// ```dart
  /// final user = const Some<String>('John').expect('the session has a user');
  /// ```
  T expect(String message) {
    return switch (this) {
      Some<T>(:final value) => value,
      None<T>() => throw StateError(message),
    };
  }
}
