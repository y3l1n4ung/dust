import '../models/order.dart';

/// Where orders live, attached with `withState`.
final class OrderStore {
  /// Creates a store holding [seed].
  OrderStore([List<Order>? seed]) : orders = [...?seed];

  /// Every order, in insertion order.
  final List<Order> orders;

  /// One order, or `null`.
  Order? find(String id) => orders.where((order) => order.id == id).firstOrNull;
}
