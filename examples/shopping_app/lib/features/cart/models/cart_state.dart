import 'package:dust_dart/derive.dart';
import 'cart_item.dart';

part 'cart_state.g.dart';

/// Notification message for cart events.
@Derive([ToString(), Eq()])
class CartNotification with _$CartNotification {
  /// Message.
  final String message;

  /// Type.
  final CartNotificationType type;

  /// Creates a [CartNotification].
  const CartNotification({required this.message, required this.type});
}

/// Cart notification type values for the shopping app example.
enum CartNotificationType {
  /// Item added cart notification type.
  itemAdded,

  /// Item removed cart notification type.
  itemRemoved,

  /// Quantity updated cart notification type.
  quantityUpdated,

  /// Cleared cart notification type.
  cleared,
}

/// Cart state for the shopping app example.
@Derive([ToString(), CopyWith(), Eq()])
class CartState with _$CartState {
  /// Items.
  final List<CartItem> items;

  /// Notification.
  final CartNotification? notification;

  /// Creates a [CartState].
  const CartState({this.items = const [], this.notification});

  /// Item count.
  int get itemCount => items.fold(0, (sum, item) => sum + item.quantity);

  /// Total price.
  double get totalPrice =>
      items.fold(0.0, (sum, item) => sum + item.totalPrice);

  /// Clear notification
  CartState clearNotification() => CartState(items: items);
}
