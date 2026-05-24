import 'package:dust_state/dust_state.dart';

import '../../cart/models/cart_item.dart';
import '../models/order.dart';
import '../models/orders_state.dart';

part 'orders_view_model.g.dart';

final class OrdersViewModelArgs extends ViewModelArgs {
  const OrdersViewModelArgs({super.observer});
}

@ViewModel(state: OrdersState, args: OrdersViewModelArgs)
class OrdersViewModel extends $OrdersViewModel {
  OrdersViewModel(super.args);

  void placeOrder({
    required List<CartItem> items,
    required double totalAmount,
    required ShippingAddress shippingAddress,
  }) {
    final order = Order(
      id: DateTime.now().millisecondsSinceEpoch.toString(),
      items: items,
      totalAmount: totalAmount,
      status: OrderStatus.pending,
      createdAt: DateTime.now(),
      shippingAddress: shippingAddress,
    );

    emit(state.copyWith(orders: [order, ...state.orders]));
  }

  void updateOrderStatus(String orderId, OrderStatus status) {
    final updatedOrders = state.orders.map((order) {
      if (order.id == orderId) {
        return order.copyWith(status: status);
      }
      return order;
    }).toList();

    emit(state.copyWith(orders: updatedOrders));
  }
}
