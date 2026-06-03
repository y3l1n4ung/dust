import 'package:dust_dart/serde.dart';

part 'user.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class User with _$User {
  const User({
    required this.id,
    required this.email,
    required this.username,
    required this.name,
    required this.phone,
  });

  final int id;
  final String email;
  final String username;
  final Name name;
  final String phone;

  factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
}

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class Name with _$Name {
  const Name({required this.firstname, required this.lastname});

  final String firstname;
  final String lastname;

  String get fullName => '$firstname $lastname';

  factory Name.fromJson(Map<String, Object?> json) => _$NameFromJson(json);
}
