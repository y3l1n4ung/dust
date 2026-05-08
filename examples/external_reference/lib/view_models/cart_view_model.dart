import 'package:ease_annotation/ease_annotation.dart';
import 'package:ease_state_helper/ease_state_helper.dart';

import '../models/product.dart';

part 'cart_view_model.ease.dart';

/// Shopping cart status
class CartStatus {
  final List<CartItem> items;
  final bool isLoading;
  final String? error;

  const CartStatus({
    this.items = const [],
    this.isLoading = false,
    this.error,
  });

  int get itemCount => items.fold(0, (sum, item) => sum + item.quantity);
  double get subtotal => items.fold(0, (sum, item) => sum + item.total);
  double get tax => subtotal * 0.1; // 10% tax
  double get total => subtotal + tax;
  bool get isEmpty => items.isEmpty;

  CartStatus copyWith({
    List<CartItem>? items,
    bool? isLoading,
    String? error,
  }) {
    return CartStatus(
      items: items ?? this.items,
      isLoading: isLoading ?? this.isLoading,
      error: error,
    );
  }
}

/// Cart ViewModel - demonstrates shopping cart with computed values
@ease
class CartViewModel extends StateNotifier<CartStatus> {
  CartViewModel() : super(const CartStatus());

  /// Add product to cart
  void addToCart(Product product) {
    final existingIndex = state.items.indexWhere(
      (item) => item.product.id == product.id,
    );

    if (existingIndex >= 0) {
      // Increase quantity
      final updatedItems = [...state.items];
      updatedItems[existingIndex] = updatedItems[existingIndex].copyWith(
        quantity: updatedItems[existingIndex].quantity + 1,
      );
      state = state.copyWith(items: updatedItems);
    } else {
      // Add new item
      state = state.copyWith(
        items: [...state.items, CartItem(product: product)],
      );
    }
  }

  /// Remove product from cart
  void removeFromCart(String productId) {
    state = state.copyWith(
      items: state.items.where((item) => item.product.id != productId).toList(),
    );
  }

  /// Update item quantity
  void updateQuantity(String productId, int quantity) {
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

    state = state.copyWith(items: updatedItems);
  }

  /// Increment quantity
  void incrementQuantity(String productId) {
    final item = state.items.firstWhere(
      (item) => item.product.id == productId,
      orElse: () => throw Exception('Item not found'),
    );
    updateQuantity(productId, item.quantity + 1);
  }

  /// Decrement quantity
  void decrementQuantity(String productId) {
    final item = state.items.firstWhere(
      (item) => item.product.id == productId,
      orElse: () => throw Exception('Item not found'),
    );
    updateQuantity(productId, item.quantity - 1);
  }

  /// Clear cart
  void clearCart() {
    state = const CartStatus();
  }

  /// Simulate checkout
  Future<bool> checkout() async {
    state = state.copyWith(isLoading: true, error: null);

    // Simulate API call
    await Future.delayed(const Duration(seconds: 2));

    // Simulate success
    state = const CartStatus();
    return true;
  }
}
