import 'package:dust_dart/serde.dart';

part 'order.g.dart';

/// One placed order.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
final class Order with _$Order {
  /// Creates an [Order].
  const Order({
    required this.id,
    required this.item,
    required this.quantity,
  });

  /// Reads an [Order] from decoded JSON.
  static Order deserialize(Map<String, Object?> json) =>
      _$OrderDeserialize(json);

  /// The primary key.
  final String id;

  /// What was ordered.
  final String item;

  /// How many.
  final int quantity;
}
