import 'order.dart';

class OrdersState {
  final List<Order> orders;
  final bool isLoading;

  const OrdersState({this.orders = const [], this.isLoading = false});

  OrdersState copyWith({List<Order>? orders, bool? isLoading}) {
    return OrdersState(
      orders: orders ?? this.orders,
      isLoading: isLoading ?? this.isLoading,
    );
  }
}
