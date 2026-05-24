import '../../orders/models/order.dart';
import 'checkout_quote.dart';

enum CheckoutStatus { initial, processing, success, error }

class CheckoutState {
  final CheckoutStatus status;
  final ShippingAddress? shippingAddress;
  final String? errorMessage;
  final String? orderId;
  final String? couponCode;
  final CheckoutQuote? quote;
  final bool isQuoteLoading;

  const CheckoutState({
    this.status = CheckoutStatus.initial,
    this.shippingAddress,
    this.errorMessage,
    this.orderId,
    this.couponCode,
    this.quote,
    this.isQuoteLoading = false,
  });

  CheckoutState copyWith({
    CheckoutStatus? status,
    ShippingAddress? shippingAddress,
    String? errorMessage,
    String? orderId,
    String? couponCode,
    CheckoutQuote? quote,
    bool? isQuoteLoading,
  }) {
    return CheckoutState(
      status: status ?? this.status,
      shippingAddress: shippingAddress ?? this.shippingAddress,
      errorMessage: errorMessage ?? this.errorMessage,
      orderId: orderId ?? this.orderId,
      couponCode: couponCode ?? this.couponCode,
      quote: quote ?? this.quote,
      isQuoteLoading: isQuoteLoading ?? this.isQuoteLoading,
    );
  }
}
