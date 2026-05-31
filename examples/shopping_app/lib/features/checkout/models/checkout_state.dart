import 'package:derive_annotation/derive_annotation.dart';
import '../../orders/models/order.dart';
import 'checkout_quote.dart';

part 'checkout_state.g.dart';

enum CheckoutStatus { initial, processing, success, error }

@Derive([ToString(), CopyWith(), Eq()])
class CheckoutState with _$CheckoutState {
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
}
