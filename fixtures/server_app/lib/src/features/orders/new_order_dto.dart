import 'package:dust_dart/serde.dart';

part 'new_order_dto.g.dart';

/// What `POST /orders` accepts.
@Derive([ToString(), Eq(), Deserialize(), Validate()])
final class NewOrder with _$NewOrder {
  /// Creates a [NewOrder].
  const NewOrder({required this.item, this.quantity = 1});

  /// Reads a [NewOrder] from decoded JSON.
  static NewOrder deserialize(Map<String, Object?> json) =>
      _$NewOrderDeserialize(json);

  /// What is being ordered.
  @Validate(length: Length(min: 1), message: 'is required')
  final String item;

  /// How many, capped so one request cannot drain the shelf.
  @Validate(range: Range(min: 1, max: 10), message: 'must be 1 to 10')
  @SerDe(defaultValue: 1)
  final int quantity;
}
