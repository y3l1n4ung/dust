import 'package:dust_dart/derive.dart';
import '../../orders/models/order.dart';
import 'checkout_quote.dart';

part 'checkout_state.g.dart';

/// Checkout status values for the shopping app example.
enum CheckoutStatus {
  /// Initial checkout status.
  initial,

  /// Processing checkout status.
  processing,

  /// Success checkout status.
  success,

  /// Error checkout status.
  error,
}

/// Checkout state for the shopping app example.
@Derive([ToString(), CopyWith(), Eq()])
class CheckoutState with _$CheckoutState {
  /// Status.
  final CheckoutStatus status;

  /// Shipping address.
  final ShippingAddress? shippingAddress;

  /// Error message.
  final String? errorMessage;

  /// Order ID.
  final String? orderId;

  /// Coupon code.
  final String? couponCode;

  /// Quote.
  final CheckoutQuote? quote;

  /// Is quote loading.
  final bool isQuoteLoading;

  /// Creates a [CheckoutState].
  const CheckoutState({
    this.status = CheckoutStatus.initial,
    this.shippingAddress,
    this.errorMessage,
    this.orderId,
    this.couponCode,
    this.quote,
    this.isQuoteLoading = false,
  });
}
