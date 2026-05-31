import 'package:derive_annotation/derive_annotation.dart';
import 'cart_item.dart';

part 'cart_state.g.dart';

/// Notification message for cart events.
@Derive([ToString(), Eq()])
class CartNotification with _$CartNotification {
  final String message;
  final CartNotificationType type;

  const CartNotification({required this.message, required this.type});
}

enum CartNotificationType { itemAdded, itemRemoved, quantityUpdated, cleared }

@Derive([ToString(), CopyWith(), Eq()])
class CartState with _$CartState {
  final List<CartItem> items;
  final CartNotification? notification;

  const CartState({this.items = const [], this.notification});

  int get itemCount => items.fold(0, (sum, item) => sum + item.quantity);

  double get totalPrice =>
      items.fold(0.0, (sum, item) => sum + item.totalPrice);

  /// Clear notification
  CartState clearNotification() => CartState(items: items);
}
