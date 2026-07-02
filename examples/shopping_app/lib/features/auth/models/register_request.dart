import 'package:dust_dart/serde.dart';

part 'register_request.g.dart';

/// Register request model for the shopping app example.
@Derive([Validate()])
class RegisterRequest with _$RegisterRequest {
  /// Creates a [RegisterRequest].
  const RegisterRequest({
    required this.email,
    required this.username,
    required this.password,
    required this.confirmPassword,
    required this.firstName,
    required this.lastName,
    required this.phone,
  });

  /// Email.
  @Validate(length: Length(min: 1), message: 'Please enter email')
  @Validate(email: true, message: 'Please enter a valid email')
  final String email;

  /// Username.
  @Validate(length: Length(min: 1), message: 'Please enter username')
  @Validate(
    length: Length(min: 3),
    message: 'Username must be at least 3 characters',
  )
  final String username;

  /// Password.
  @Validate(length: Length(min: 1), message: 'Please enter password')
  @Validate(
    length: Length(min: 6),
    message: 'Password must be at least 6 characters',
  )
  final String password;

  /// Confirm password.
  @Validate(length: Length(min: 1), message: 'Please confirm password')
  @Validate(mustMatch: 'password', message: 'Passwords do not match')
  final String confirmPassword;

  /// First name.
  @Validate(length: Length(min: 1), message: 'Required')
  final String firstName;

  /// Last name.
  @Validate(length: Length(min: 1), message: 'Required')
  final String lastName;

  /// Phone.
  @Validate(length: Length(min: 1), message: 'Please enter phone number')
  final String phone;
}
