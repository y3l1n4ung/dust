/// Base marker for all Dust derive annotations.
abstract base class DeriveMeta {
  /// Creates one derive metadata marker.
  const DeriveMeta();
}

/// Base marker for derive traits listed inside `@Derive([...])`.
abstract base class DeriveTrait extends DeriveMeta {
  /// Creates one derive trait marker.
  const DeriveTrait();
}

/// Base marker for derive configuration annotations.
abstract base class DeriveConfig extends DeriveMeta {
  /// Creates one derive configuration annotation.
  const DeriveConfig();
}

/// Declares which traits Dust should generate for the annotated declaration.
final class Derive extends DeriveMeta {
  /// The requested derive traits.
  final List<DeriveTrait> traits;

  /// Creates one derive request.
  const Derive(this.traits);
}
