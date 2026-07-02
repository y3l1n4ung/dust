import 'package:dust_dart/serde.dart';

part 'store_cart.g.dart';

/// Store cart model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class StoreCart with _$StoreCart {
  /// Creates a [StoreCart].
  const StoreCart({
    required this.id,
    required this.userId,
    required this.date,
    required this.products,
  });

  /// Unique identifier.
  final int id;

  /// User ID.
  final int userId;

  /// Date.
  final DateTime date;

  /// Products.
  final List<StoreCartProduct> products;

  /// Item count.
  int get itemCount => products.fold(0, (sum, item) => sum + item.quantity);

  /// Creates a [StoreCart] from JSON.
  factory StoreCart.fromJson(Map<String, Object?> json) =>
      _$StoreCartFromJson(json);
}

/// Store cart product model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class StoreCartProduct with _$StoreCartProduct {
  /// Creates a [StoreCartProduct].
  const StoreCartProduct({required this.productId, required this.quantity});

  /// Product ID.
  final int productId;

  /// Quantity.
  final int quantity;

  /// Creates a [StoreCartProduct] from JSON.
  factory StoreCartProduct.fromJson(Map<String, Object?> json) =>
      _$StoreCartProductFromJson(json);
}
