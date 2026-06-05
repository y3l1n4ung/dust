/// Empty success value for operations that only signal completion.
final class Unit {
  /// Creates one unit value.
  const Unit();

  @override
  bool operator ==(Object other) => other is Unit;

  @override
  int get hashCode => 0;

  @override
  String toString() => 'unit';
}

/// Shared unit value.
const unit = Unit();
