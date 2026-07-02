import 'package:dust_flutter/state.dart';

import '../../../core/data/shopping_repository.dart';
import '../../cart/models/cart_item.dart';
import '../../orders/models/order.dart';
import '../models/checkout_quote.dart';
import '../models/checkout_state.dart';

part 'checkout_view_model.g.dart';

/// Checkout view model args model for the shopping app example.
final class CheckoutViewModelArgs extends ViewModelArgs {
  /// Creates a [CheckoutViewModelArgs].
  const CheckoutViewModelArgs({required this.repository, super.observer});

  /// Repository.
  final ShoppingRepository repository;
}

/// Checkout view model for the shopping app example.
@ViewModel(state: CheckoutState, args: CheckoutViewModelArgs)
class CheckoutViewModel extends $CheckoutViewModel {
  /// Creates a [CheckoutViewModel].
  CheckoutViewModel(super.args);

  /// Updates shipping address.
  void updateShippingAddress(ShippingAddress address) {
    emit(state.copyWith(shippingAddress: address));
  }

  /// Applies coupon.
  Future<void> applyCoupon({
    required double subtotal,
    required String couponCode,
  }) async {
    final code = couponCode.trim();
    emit(state.copyWith(couponCode: code, isQuoteLoading: true));

    final quote = await args.repository.quoteCheckout(
      CheckoutQuoteRequest(subtotal: subtotal, couponCode: code),
    );
    emit(state.copyWith(quote: quote, isQuoteLoading: false));
  }

  /// Processes checkout.
  Future<String?> processCheckout(
    List<CartItem> items,
    double totalAmount,
  ) async {
    if (state.shippingAddress == null) {
      emit(
        state.copyWith(
          status: CheckoutStatus.error,
          errorMessage: 'Please enter shipping address',
        ),
      );
      return null;
    }

    emit(state.copyWith(status: CheckoutStatus.processing));

    await Future<void>.delayed(const Duration(milliseconds: 600));

    final orderId = DateTime.now().millisecondsSinceEpoch.toString();
    emit(state.copyWith(status: CheckoutStatus.success, orderId: orderId));

    return orderId;
  }

  /// Resets state.
  void reset() {
    invalidateSelf();
  }
}
