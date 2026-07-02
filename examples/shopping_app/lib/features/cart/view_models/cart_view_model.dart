import 'package:dust_flutter/state.dart';

import '../../../core/logging/logger.dart';
import '../../products/models/product.dart';
import '../models/cart_item.dart';
import '../models/cart_state.dart';

part 'cart_view_model.g.dart';

/// Cart view model args model for the shopping app example.
final class CartViewModelArgs extends ViewModelArgs {
  /// Creates a [CartViewModelArgs].
  const CartViewModelArgs({super.observer});
}

/// Cart view model for the shopping app example.
@ViewModel(state: CartState, args: CartViewModelArgs)
class CartViewModel extends $CartViewModel {
  /// Creates a [CartViewModel].
  CartViewModel(super.args);

  /// Adds to cart.
  void addToCart(Product product) {
    final existingIndex = state.items.indexWhere(
      (item) => item.product.id == product.id,
    );

    if (existingIndex >= 0) {
      final updatedItems = List<CartItem>.from(state.items);
      final existing = updatedItems[existingIndex];
      updatedItems[existingIndex] = existing.copyWith(
        quantity: existing.quantity + 1,
      );
      emit(state.copyWith(items: updatedItems));
      emitEffect(
        CartNotification(
          message: '${product.title} quantity updated',
          type: CartNotificationType.quantityUpdated,
        ),
      );
    } else {
      emit(
        state.copyWith(
          items: [
            ...state.items,
            CartItem(product: product, quantity: 1),
          ],
        ),
      );
      emitEffect(
        CartNotification(
          message: '${product.title} added to cart',
          type: CartNotificationType.itemAdded,
        ),
      );
    }
  }

  /// Removes from cart.
  void removeFromCart(int productId) {
    logger.userAction('remove_from_cart', {'productId': productId});
    final item = state.items.firstWhere((i) => i.product.id == productId);
    emit(
      state.copyWith(
        items: state.items.where((i) => i.product.id != productId).toList(),
      ),
    );
    emitEffect(
      CartNotification(
        message: '${item.product.title} removed',
        type: CartNotificationType.itemRemoved,
      ),
    );
    logger.info(
      'CART',
      'Removed product $productId, cart now has ${state.itemCount} items',
    );
  }

  /// Updates quantity.
  void updateQuantity(int productId, int quantity) {
    logger.userAction('update_cart_quantity', {
      'productId': productId,
      'quantity': quantity,
    });

    if (quantity <= 0) {
      removeFromCart(productId);
      return;
    }

    final updatedItems = state.items.map((item) {
      if (item.product.id == productId) {
        return item.copyWith(quantity: quantity);
      }
      return item;
    }).toList();

    emit(state.copyWith(items: updatedItems));
    logger.debug('CART', 'Updated product $productId quantity to $quantity');
  }

  /// Clears cart.
  void clearCart() {
    invalidateSelf();
    emitEffect(
      const CartNotification(
        message: 'Cart cleared',
        type: CartNotificationType.cleared,
      ),
    );
  }

  /// Clears notification.
  void clearNotification() {}
}
