import 'base.dart';

/// Generates a `toString()` implementation from the class fields.
final class ToString extends DeriveTrait {
  /// Creates the `ToString` derive marker.
  const ToString();
}

/// Deprecated alias for `ToString()`.
@Deprecated('Use ToString() instead.')
final class Debug extends DeriveTrait {
  /// Creates the legacy `Debug` derive marker.
  const Debug();
}

/// Generates typed `copyWith` support.
///
/// Generated `copyWith` uses Freezed-inspired callable ergonomics, keeps normal
/// copy operations shallow, and exposes chained helpers for nested Dust model
/// fields.
///
/// ```dart
/// final renamed = profile.copyWith(name: 'John');
/// final cleared = profile.copyWith(nickname: null);
/// final moved = profile.copyWith.address(city: 'London');
/// ```
final class CopyWith extends DeriveTrait {
  /// Creates the `CopyWith` derive marker.
  const CopyWith();
}

/// Generates value equality through `operator ==` and matching `hashCode`.
final class Eq extends DeriveTrait {
  /// Creates the `Eq` derive marker.
  const Eq();
}
