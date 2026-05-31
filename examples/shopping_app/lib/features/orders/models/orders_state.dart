import 'package:derive_annotation/derive_annotation.dart';
import 'order.dart';

part 'orders_state.g.dart';

@Derive([ToString(), CopyWith(), Eq()])
class OrdersState with _$OrdersState {
  final List<Order> orders;
  final bool isLoading;

  const OrdersState({this.orders = const [], this.isLoading = false});
}
