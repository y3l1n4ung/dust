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
