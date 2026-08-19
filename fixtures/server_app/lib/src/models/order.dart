import 'package:dust_dart/db.dart';
import 'package:dust_dart/serde.dart';

part 'order.g.dart';

/// One placed order.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize(), FromRow()])
final class Order with _$Order {
  /// Creates an [Order].
  const Order({
    required this.id,
    required this.accountId,
    required this.item,
    required this.quantity,
    required this.placedAt,
  });

  /// Reads an [Order] from decoded JSON.
  static Order deserialize(Map<String, Object?> json) =>
      _$OrderDeserialize(json);

  /// The primary key.
  final int id;

  /// Who placed it. Every query filters on this — an order is not readable by
  /// whoever guesses its id.
  @Sqlx(rename: 'account_id')
  final int accountId;

  /// What was ordered.
  final String item;

  /// How many.
  final int quantity;

  /// When it was placed, ISO-8601 in UTC.
  @Sqlx(rename: 'placed_at')
  final String placedAt;
}
