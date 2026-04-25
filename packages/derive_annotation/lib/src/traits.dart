import 'base.dart';

/// Generates a readable debug-oriented `toString()` implementation.
final class Debug extends DeriveTrait {
  /// Creates the `Debug` derive marker.
  const Debug();
}

/// Generates value cloning support.
final class Clone extends DeriveTrait {
  /// Creates the `Clone` derive marker.
  const Clone();
}

/// Generates `copyWith(...)` support.
final class CopyWith extends DeriveTrait {
  /// Creates the `CopyWith` derive marker.
  const CopyWith();
}

/// Generates value equality through `operator ==`.
///
/// In the current Dart backend this is emitted the same way as [Eq], because
/// Dart exposes a single equality operator.
final class PartialEq extends DeriveTrait {
  /// Creates the `PartialEq` derive marker.
  const PartialEq();
}

/// Marks the type as having strong value equality semantics.
///
/// In the current Dart backend this is emitted the same way as [PartialEq].
final class Eq extends DeriveTrait {
  /// Creates the `Eq` derive marker.
  const Eq();
}

/// Generates `hashCode`.
final class Hash extends DeriveTrait {
  /// Creates the `Hash` derive marker.
  const Hash();
}
