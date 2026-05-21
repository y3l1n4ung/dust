import 'package:derive_serde_annotation/derive_serde_annotation.dart';

import '../../cart/models/cart_item.dart';

part 'order.g.dart';

@Derive([Serialize(), Deserialize()])
enum OrderStatus { pending, processing, shipped, delivered, cancelled }

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class Order with _$OrderDust {
  const Order({
    required this.id,
    required this.items,
    required this.totalAmount,
    required this.status,
    required this.createdAt,
    required this.shippingAddress,
  });

  final String id;
  final List<CartItem> items;
  final double totalAmount;
  final OrderStatus status;
  final DateTime createdAt;
  final ShippingAddress shippingAddress;

  factory Order.fromJson(Map<String, Object?> json) => _$OrderFromJson(json);
}

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class ShippingAddress with _$ShippingAddressDust {
  const ShippingAddress({
    required this.fullName,
    required this.address,
    required this.city,
    required this.zipCode,
    required this.phone,
  });

  final String fullName;
  final String address;
  final String city;
  final String zipCode;
  final String phone;

  factory ShippingAddress.fromJson(Map<String, Object?> json) =>
      _$ShippingAddressFromJson(json);
}
