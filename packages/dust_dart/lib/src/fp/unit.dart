/// Empty success value for operations that only signal completion.
///
/// Use `Unit` when a `Result` has no meaningful success payload.
///
/// ```dart
/// Result<Unit, String> save() {
///   return const Ok(unit);
/// }
/// ```
final class Unit {
  /// Creates one unit value.
  ///
  /// Prefer the shared [unit] constant unless a const constructor is required.
  ///
  /// ```dart
  /// const value = Unit();
  /// ```
  const Unit();

  @override
  bool operator ==(Object other) => other is Unit;

  @override
  int get hashCode => 0;

  @override
  String toString() => 'unit';
}

/// Shared unit value.
///
/// ```dart
/// Result<Unit, String> saved = const Ok(unit);
/// ```
const unit = Unit();
