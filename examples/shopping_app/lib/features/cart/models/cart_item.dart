import 'package:dust_dart/serde.dart';

import '../../products/models/product.dart';

part 'cart_item.g.dart';

/// Cart item model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class CartItem with _$CartItem {
  /// Creates a [CartItem].
  const CartItem({required this.product, required this.quantity});

  /// Product.
  final Product product;

  /// Quantity.
  final int quantity;

  /// Total price.
  double get totalPrice => product.price * quantity;

  /// Creates a [CartItem] from JSON.
  factory CartItem.fromJson(Map<String, Object?> json) =>
      _$CartItemFromJson(json);
}
