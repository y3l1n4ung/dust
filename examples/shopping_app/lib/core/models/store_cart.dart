import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'store_cart.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class StoreCart with _$StoreCart {
  const StoreCart({
    required this.id,
    required this.userId,
    required this.date,
    required this.products,
  });

  final int id;
  final int userId;
  final DateTime date;
  final List<StoreCartProduct> products;

  int get itemCount => products.fold(0, (sum, item) => sum + item.quantity);

  factory StoreCart.fromJson(Map<String, Object?> json) =>
      _$StoreCartFromJson(json);
}

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class StoreCartProduct with _$StoreCartProduct {
  const StoreCartProduct({required this.productId, required this.quantity});

  final int productId;
  final int quantity;

  factory StoreCartProduct.fromJson(Map<String, Object?> json) =>
      _$StoreCartProductFromJson(json);
}
