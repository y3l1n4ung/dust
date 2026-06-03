import 'package:dust_dart/serde.dart';

part 'checkout_quote.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class CheckoutQuoteRequest with _$CheckoutQuoteRequest {
  const CheckoutQuoteRequest({required this.subtotal, this.couponCode});

  final double subtotal;
  final String? couponCode;

  factory CheckoutQuoteRequest.fromJson(Map<String, Object?> json) =>
      _$CheckoutQuoteRequestFromJson(json);
}

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class CheckoutQuote with _$CheckoutQuote {
  const CheckoutQuote({
    required this.subtotal,
    required this.discount,
    required this.shipping,
    required this.tax,
    required this.total,
    required this.estimatedDeliveryDays,
    this.appliedCoupon,
    this.message,
  });

  final double subtotal;
  final double discount;
  final double shipping;
  final double tax;
  final double total;
  final int estimatedDeliveryDays;
  final String? appliedCoupon;
  final String? message;

  factory CheckoutQuote.fromJson(Map<String, Object?> json) =>
      _$CheckoutQuoteFromJson(json);
}
