import 'package:dust_dart/serde.dart';

import '../../cart/models/cart_item.dart';

part 'order.g.dart';

/// Order status values for the shopping app example.
@Derive([Serialize(), Deserialize()])
enum OrderStatus {
  /// Pending order status.
  pending,

  /// Processing order status.
  processing,

  /// Shipped order status.
  shipped,

  /// Delivered order status.
  delivered,

  /// Cancelled order status.
  cancelled,
}

/// Order model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class Order with _$Order {
  /// Creates an [Order].
  const Order({
    required this.id,
    required this.items,
    required this.totalAmount,
    required this.status,
    required this.createdAt,
    required this.shippingAddress,
  });

  /// Unique identifier.
  final String id;

  /// Items.
  final List<CartItem> items;

  /// Total amount.
  final double totalAmount;

  /// Status.
  final OrderStatus status;

  /// Created at.
  final DateTime createdAt;

  /// Shipping address.
  final ShippingAddress shippingAddress;

  /// Creates an [Order] from JSON.
  factory Order.fromJson(Map<String, Object?> json) => _$OrderFromJson(json);
}

/// Shipping address model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class ShippingAddress with _$ShippingAddress {
  /// Creates a [ShippingAddress].
  const ShippingAddress({
    required this.fullName,
    required this.address,
    required this.city,
    required this.zipCode,
    required this.phone,
  });

  /// Full name.
  final String fullName;

  /// Address.
  final String address;

  /// City.
  final String city;

  /// Zip code.
  final String zipCode;

  /// Phone.
  final String phone;

  /// Creates a [ShippingAddress] from JSON.
  factory ShippingAddress.fromJson(Map<String, Object?> json) =>
      _$ShippingAddressFromJson(json);
}
