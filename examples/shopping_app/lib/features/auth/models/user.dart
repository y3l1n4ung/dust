import 'package:dust_dart/serde.dart';

part 'user.g.dart';

/// User model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class User with _$User {
  /// Creates an [User].
  const User({
    required this.id,
    required this.email,
    required this.username,
    required this.name,
    required this.phone,
  });

  /// Unique identifier.
  final int id;

  /// Email.
  final String email;

  /// Username.
  final String username;

  /// Name.
  final Name name;

  /// Phone.
  final String phone;

  /// Creates an [User] from JSON.
  factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
}

/// Name model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class Name with _$Name {
  /// Creates a [Name].
  const Name({required this.firstname, required this.lastname});

  /// Firstname.
  final String firstname;

  /// Lastname.
  final String lastname;

  /// Full name.
  String get fullName => '$firstname $lastname';

  /// Creates a [Name] from JSON.
  factory Name.fromJson(Map<String, Object?> json) => _$NameFromJson(json);
}
