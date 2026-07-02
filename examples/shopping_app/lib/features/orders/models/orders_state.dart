import 'package:dust_dart/derive.dart';
import 'order.dart';

part 'orders_state.g.dart';

/// Orders state for the shopping app example.
@Derive([ToString(), CopyWith(), Eq()])
class OrdersState with _$OrdersState {
  /// Orders.
  final List<Order> orders;

  /// Is loading.
  final bool isLoading;

  /// Creates an [OrdersState].
  const OrdersState({this.orders = const [], this.isLoading = false});
}
