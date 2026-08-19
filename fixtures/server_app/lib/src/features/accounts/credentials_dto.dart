import 'package:dust_dart/serde.dart';

part 'credentials_dto.g.dart';

/// What `POST /auth/tokens` accepts.
///
/// `Serialize` is deliberately not derived: this type holds a password, and a
/// type that cannot be serialized cannot be logged by a generic logger or
/// returned by a handler that meant to return something else.
@Derive([Deserialize(), Validate()])
final class Credentials with _$Credentials {
  /// Creates a [Credentials].
  const Credentials({required this.email, required this.password});

  /// Reads a [Credentials] from decoded JSON.
  static Credentials deserialize(Map<String, Object?> json) =>
      _$CredentialsDeserialize(json);

  /// Who is signing in.
  @Validate(email: true, message: 'must be an email address')
  final String email;

  /// Their password.
  ///
  /// A length floor only. Composition rules — a digit, a symbol — push people
  /// towards `Password1!` and are worse than length.
  @Validate(length: Length(min: 12), message: 'must be at least 12 characters')
  final String password;
}

/// What a successful sign-in returns.
@Derive([Serialize()])
final class IssuedToken with _$IssuedToken {
  /// Creates an [IssuedToken].
  const IssuedToken({required this.token, required this.expiresAt});

  /// The token, shown exactly once. Only its hash is stored.
  final String token;

  /// When it stops working, ISO-8601 in UTC.
  final String expiresAt;
}
