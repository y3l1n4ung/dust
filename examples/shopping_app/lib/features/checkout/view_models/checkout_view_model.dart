import 'package:dust_state/dust_state.dart';

import '../../cart/models/cart_item.dart';
import '../../orders/models/order.dart';
import '../models/checkout_state.dart';

part 'checkout_view_model.g.dart';

final class CheckoutViewModelArgs extends ViewModelArgs {
  const CheckoutViewModelArgs({super.observer});
}

@ViewModel(state: CheckoutState, args: CheckoutViewModelArgs)
class CheckoutViewModel extends $CheckoutViewModel {
  CheckoutViewModel(super.args);

  void updateShippingAddress(ShippingAddress address) {
    emit(state.copyWith(shippingAddress: address));
  }

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

  void reset() {
    emit(const CheckoutState());
  }
}

extension CheckoutViewModelContext on BuildContext {
  CheckoutViewModel get checkoutViewModel => CheckoutViewModelScope.of(this);
}
