import 'package:dust_dart/serde.dart';

part 'checkout_shipping_request.g.dart';

/// Checkout shipping request model for the shopping app example.
@Derive([Validate()])
class CheckoutShippingRequest with _$CheckoutShippingRequest {
  /// Creates a [CheckoutShippingRequest].
  const CheckoutShippingRequest({
    required this.fullName,
    required this.address,
    required this.city,
    required this.zipCode,
    required this.phone,
  });

  /// Full name.
  @Validate(length: Length(min: 1), message: 'Required')
  final String fullName;

  /// Address.
  @Validate(length: Length(min: 1), message: 'Required')
  final String address;

  /// City.
  @Validate(length: Length(min: 1), message: 'Required')
  final String city;

  /// Zip code.
  @Validate(length: Length(min: 1), message: 'Required')
  final String zipCode;

  /// Phone.
  @Validate(length: Length(min: 1), message: 'Required')
  final String phone;
}
