import 'package:derive_serde_annotation/derive_serde_annotation.dart';

import '../../products/models/product.dart';

part 'cart_item.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class CartItem with _$CartItemDust {
  const CartItem({required this.product, required this.quantity});

  final Product product;
  final int quantity;

  double get totalPrice => product.price * quantity;

  factory CartItem.fromJson(Map<String, Object?> json) =>
      _$CartItemFromJson(json);
}
