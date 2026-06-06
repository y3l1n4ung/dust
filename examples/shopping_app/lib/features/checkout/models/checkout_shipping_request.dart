import 'package:dust_dart/serde.dart';

part 'checkout_shipping_request.g.dart';

@Derive([Validate()])
class CheckoutShippingRequest with _$CheckoutShippingRequest {
  const CheckoutShippingRequest({
    required this.fullName,
    required this.address,
    required this.city,
    required this.zipCode,
    required this.phone,
  });

  @Validate(length: Length(min: 1), message: 'Required')
  final String fullName;

  @Validate(length: Length(min: 1), message: 'Required')
  final String address;

  @Validate(length: Length(min: 1), message: 'Required')
  final String city;

  @Validate(length: Length(min: 1), message: 'Required')
  final String zipCode;

  @Validate(length: Length(min: 1), message: 'Required')
  final String phone;
}
