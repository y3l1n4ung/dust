import 'cart_item.dart';

/// Notification message for cart events.
class CartNotification {
  final String message;
  final CartNotificationType type;

  const CartNotification({required this.message, required this.type});
}

enum CartNotificationType { itemAdded, itemRemoved, quantityUpdated, cleared }

class CartState {
  final List<CartItem> items;
  final CartNotification? notification;

  const CartState({this.items = const [], this.notification});

  int get itemCount => items.fold(0, (sum, item) => sum + item.quantity);

  double get totalPrice =>
      items.fold(0.0, (sum, item) => sum + item.totalPrice);

  CartState copyWith({List<CartItem>? items, CartNotification? notification}) {
    return CartState(items: items ?? this.items, notification: notification);
  }

  /// Clear notification
  CartState clearNotification() => CartState(items: items);
}
