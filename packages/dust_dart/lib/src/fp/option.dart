/// Optional value wrapper for Rust-style present-or-absent values.
///
/// Use `None()` when no value is present, and `Some(value)` when a value is
/// present even if that value is `null`.
///
/// ```dart
/// const nickname = Some<String?>(null);
///
/// final label = switch (nickname) {
///   None<String?>() => 'missing',
///   Some<String?>(:final value) => value ?? 'present null',
/// };
/// ```
///
/// Build an option as the type it is read through. An `Option<num>` holding
/// `None<int>()` compiles, but [unwrapOr] and [unwrapOrElse] then reject a
/// `double` fallback with a [TypeError]. The other members are safe.
sealed class Option<T> {
  /// Creates one option value.
  ///
  /// Subclasses are usually created with `None()` or `Some(...)`.
  ///
  /// ```dart
  /// const option = None<String?>();
  /// ```
  const Option();

  /// Wraps a nullable value: `null` becomes [None], anything else [Some].
  ///
  /// This is the one place a Dart `null` turns into absence. Build a
  /// `Some<T?>(null)` directly when `null` is a present value you want to
  /// keep, such as a field explicitly cleared rather than left out.
  ///
  /// ```dart
  /// final missing = Option<String>.fromNullable(null); // None()
  /// final present = Option<String>.fromNullable('John'); // Some(John)
  /// ```
  factory Option.fromNullable(T? value) {
    return value == null ? None<T>() : Some<T>(value);
  }

  /// Whether this option is [Some].
  ///
  /// ```dart
  /// const option = Some<int>(1);
  /// final hasValue = option.isSome; // true
  /// ```
  bool get isSome;

  /// Whether this option is [None].
  ///
  /// ```dart
  /// const option = None<int>();
  /// final missing = option.isNone; // true
  /// ```
  bool get isNone => !isSome;

  /// Maps a present value and leaves absent values unchanged.
  ///
  /// ```dart
  /// final doubled = const Some<int>(2).map((value) => value * 2);
  /// final missing = const None<int>().map((value) => value * 2);
  /// ```
  Option<R> map<R>(R Function(T value) mapper);

  /// Chains another option-producing operation when a value is present.
  ///
  /// ```dart
  /// Option<int> parseCount(String text) {
  ///   final value = int.tryParse(text);
  ///   return value == null ? const None<int>() : Some(value);
  /// }
  ///
  /// final parsed = const Some<String>('42').andThen(parseCount);
  /// ```
  Option<R> andThen<R>(Option<R> Function(T value) next);

  /// Returns the present value, or [fallback] when this option is absent.
  ///
  /// Throws a [TypeError] when this value was built with a narrower type than
  /// the one it is read through; see [Option].
  ///
  /// ```dart
  /// final count = const None<int>().unwrapOr(0);
  /// final existing = const Some<int>(7).unwrapOr(0);
  /// ```
  T unwrapOr(T fallback);

  /// Returns the present value, or computes one when this option is absent.
  ///
  /// Throws a [TypeError] when this value was built with a narrower type than
  /// the one it is read through; see [Option].
  ///
  /// ```dart
  /// final count = const None<int>().unwrapOrElse(() => 0);
  /// final existing = const Some<int>(7).unwrapOrElse(() => 0);
  /// ```
  T unwrapOrElse(T Function() fallback);

  /// Pattern matches this option.
  ///
  /// ```dart
  /// final label = const Some<int>(42).match(
  ///   some: (value) => 'count=$value',
  ///   none: () => 'missing',
  /// );
  /// ```
  R match<R>({
    required R Function(T value) some,
    required R Function() none,
  });
}

/// Option state for an absent value.
///
/// Match this state when no value is present.
///
/// ```dart
/// const option = None<String?>();
///
/// final label = switch (option) {
///   None<String?>() => 'Anonymous',
///   Some<String?>(:final value) => value,
/// };
/// ```
final class None<T> extends Option<T> {
  /// Creates an absent option state.
  ///
  /// ```dart
  /// const option = None<String?>();
  /// ```
  const None();

  @override
  bool get isSome => false;

  @override
  Option<R> map<R>(R Function(T value) mapper) {
    return None<R>();
  }

  @override
  Option<R> andThen<R>(Option<R> Function(T value) next) {
    return None<R>();
  }

  @override
  T unwrapOr(T fallback) => fallback;

  @override
  T unwrapOrElse(T Function() fallback) => fallback();

  @override
  R match<R>({
    required R Function(T value) some,
    required R Function() none,
  }) {
    return none();
  }

  @override
  bool operator ==(Object other) {
    return other is None<Object?>;
  }

  @override
  int get hashCode => Object.hash(None, 0);

  @override
  String toString() => 'None()';
}

/// Option state for a present value.
///
/// Match this state to read the present value.
///
/// ```dart
/// const option = Some<String?>('John');
///
/// final label = switch (option) {
///   None<String?>() => 'Anonymous',
///   Some<String?>(:final value) => value ?? 'No nickname',
/// };
/// ```
final class Some<T> extends Option<T> {
  /// Creates a present option state.
  ///
  /// ```dart
  /// const option = Some<String?>('John');
  /// ```
  const Some(this.value);

  /// Replacement value, including `null` for nullable fields.
  ///
  /// ```dart
  /// const option = Some<String?>(null);
  /// final value = option.value; // null
  /// ```
  final T value;

  @override
  bool get isSome => true;

  @override
  Option<R> map<R>(R Function(T value) mapper) {
    return Some<R>(mapper(value));
  }

  @override
  Option<R> andThen<R>(Option<R> Function(T value) next) {
    return next(value);
  }

  @override
  T unwrapOr(T fallback) => value;

  @override
  T unwrapOrElse(T Function() fallback) => value;

  @override
  R match<R>({
    required R Function(T value) some,
    required R Function() none,
  }) {
    return some(value);
  }

  @override
  bool operator ==(Object other) {
    return other is Some<Object?> && other.value == value;
  }

  @override
  int get hashCode => Object.hash(Some, value);

  @override
  String toString() => 'Some($value)';
}
