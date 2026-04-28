import 'base.dart';

/// Generates a readable debug-oriented `toString()` implementation.
final class Debug extends DeriveTrait {
  /// Creates the `Debug` derive marker.
  const Debug();
}

/// Generates `copyWith(...)` support.
final class CopyWith extends DeriveTrait {
  /// Creates the `CopyWith` derive marker.
  const CopyWith();
}

/// Generates value equality through `operator ==` and matching `hashCode`.
final class Eq extends DeriveTrait {
  /// Creates the `Eq` derive marker.
  const Eq();
}
