import '../../orders/models/order.dart';

enum CheckoutStatus { initial, processing, success, error }

class CheckoutState {
  final CheckoutStatus status;
  final ShippingAddress? shippingAddress;
  final String? errorMessage;
  final String? orderId;

  const CheckoutState({
    this.status = CheckoutStatus.initial,
    this.shippingAddress,
    this.errorMessage,
    this.orderId,
  });

  CheckoutState copyWith({
    CheckoutStatus? status,
    ShippingAddress? shippingAddress,
    String? errorMessage,
    String? orderId,
  }) {
    return CheckoutState(
      status: status ?? this.status,
      shippingAddress: shippingAddress ?? this.shippingAddress,
      errorMessage: errorMessage ?? this.errorMessage,
      orderId: orderId ?? this.orderId,
    );
  }
}
