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
/// Generated nullable `copyWith` methods use an outer `Option` as an update
/// envelope: `None()` keeps the previous field value, while `Some(null)`
/// clears a nullable field.
///
/// ```dart
/// user.copyWith(nickname: const Some<String?>(null));
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
/// Match this state when no value is present. In generated nullable
/// `copyWith` parameters, this is the outer update envelope state for keeping
/// the existing field value.
///
/// ```dart
/// const option = None<String?>();
///
/// final next = switch (option) {
///   None<String?>() => currentNickname,
///   Some<String?>(:final value) => value,
/// };
/// ```
final class None<T> extends Option<T> {
  /// Creates an absent option state.
  ///
  /// Use this as the default value for nullable generated `copyWith`
  /// parameters so omitted arguments keep the current field value.
  ///
  /// ```dart
  /// const option = None<String?>();
  /// ```
  const None();
}

/// Option state for a present value.
///
/// Match this state to read the present value. In generated nullable
/// `copyWith` parameters, this carries the replacement value.
///
/// ```dart
/// const option = Some<String?>('Ye');
///
/// final next = switch (option) {
///   None<String?>() => currentNickname,
///   Some<String?>(:final value) => value,
/// };
/// ```
final class Some<T> extends Option<T> {
  /// Creates a present option state.
  ///
  /// In generated nullable `copyWith` parameters, this carries the replacement
  /// value. Pass `null` there when the field should be cleared.
  ///
  /// ```dart
  /// const option = Some<String?>('Ye');
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
