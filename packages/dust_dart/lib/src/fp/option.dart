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
sealed class Option<T> {
  /// Creates one option value.
  ///
  /// Subclasses are usually created with `None()` or `Some(...)`.
  ///
  /// ```dart
  /// const option = None<String?>();
  /// ```
  const Option();
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
}
