import 'package:dust_dart/serde.dart';

part 'checkout_quote.g.dart';

/// Checkout quote request model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class CheckoutQuoteRequest with _$CheckoutQuoteRequest {
  /// Creates a [CheckoutQuoteRequest].
  const CheckoutQuoteRequest({required this.subtotal, this.couponCode});

  /// Subtotal.
  final double subtotal;

  /// Coupon code.
  final String? couponCode;

  /// Creates a [CheckoutQuoteRequest] from JSON.
  factory CheckoutQuoteRequest.fromJson(Map<String, Object?> json) =>
      _$CheckoutQuoteRequestFromJson(json);
}

/// Checkout quote model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class CheckoutQuote with _$CheckoutQuote {
  /// Creates a [CheckoutQuote].
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

  /// Subtotal.
  final double subtotal;

  /// Discount.
  final double discount;

  /// Shipping.
  final double shipping;

  /// Tax.
  final double tax;

  /// Total.
  final double total;

  /// Estimated delivery days.
  final int estimatedDeliveryDays;

  /// Applied coupon.
  final String? appliedCoupon;

  /// Message.
  final String? message;

  /// Creates a [CheckoutQuote] from JSON.
  factory CheckoutQuote.fromJson(Map<String, Object?> json) =>
      _$CheckoutQuoteFromJson(json);
}
