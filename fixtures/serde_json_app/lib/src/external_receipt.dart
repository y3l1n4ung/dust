/// External receipt model for the serde fixture app.
final class ExternalReceipt {
  /// Creates an [ExternalReceipt].
  const ExternalReceipt({required this.id, required this.cents});

  /// Creates an [ExternalReceipt] from JSON.
  factory ExternalReceipt.fromJson(Map<String, Object?> json) {
    return ExternalReceipt(
      id: json['id'] as String,
      cents: json['cents'] as int,
    );
  }

  /// Unique identifier.
  final String id;

  /// Cents.
  final int cents;

  /// Converts this value to JSON.
  Map<String, Object?> toJson() {
    return <String, Object?>{'id': id, 'cents': cents};
  }
}
