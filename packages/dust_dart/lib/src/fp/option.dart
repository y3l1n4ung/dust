/// Optional value wrapper used when `null` is a meaningful value.
///
/// Use `None()` when a value should stay unchanged, and `Some(value)` when a
/// value should be present even if that value is `null`.
///
/// ```dart
/// const nickname = Some<String?>(null);
///
/// final label = switch (nickname) {
///   None<String?>() => 'unchanged',
///   Some<String?>(:final value) => value ?? 'cleared',
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

/// Option state that leaves the current value unchanged.
///
/// Match this state when generated code should keep the existing field value.
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
  /// Creates an unchanged option state.
  ///
  /// Use this as the default value for nullable generated `copyWith`
  /// parameters.
  ///
  /// ```dart
  /// const option = None<String?>();
  /// ```
  const None();
}

/// Option state that replaces the current value.
///
/// Match this state to read the replacement value.
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
  /// Creates a replacement option state.
  ///
  /// Pass `null` when the nullable generated `copyWith` field should be
  /// cleared.
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
