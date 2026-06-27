final class ExternalReceipt {
  const ExternalReceipt({required this.id, required this.cents});

  factory ExternalReceipt.fromJson(Map<String, Object?> json) {
    return ExternalReceipt(
      id: json['id'] as String,
      cents: json['cents'] as int,
    );
  }

  final String id;
  final int cents;

  Map<String, Object?> toJson() {
    return <String, Object?>{'id': id, 'cents': cents};
  }
}
