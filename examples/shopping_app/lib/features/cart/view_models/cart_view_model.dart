import 'package:dust_state/dust_state.dart';

import '../../../core/logging/logger.dart';
import '../../products/models/product.dart';
import '../models/cart_item.dart';
import '../models/cart_state.dart';

part 'cart_view_model.g.dart';

final class CartViewModelArgs extends ViewModelArgs {
  const CartViewModelArgs({super.observer});
}

@ViewModel(state: CartState, args: CartViewModelArgs)
class CartViewModel extends $CartViewModel {
  CartViewModel(super.args);

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

  void clearCart() {
    emit(const CartState());
    emitEffect(
      const CartNotification(
        message: 'Cart cleared',
        type: CartNotificationType.cleared,
      ),
    );
  }

  void clearNotification() {}
}
