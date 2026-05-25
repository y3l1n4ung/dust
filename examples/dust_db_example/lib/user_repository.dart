import 'dart:convert';

import 'package:dust_db/dust_db.dart';

part 'user_repository.g.dart';

@FromRow()
@Sqlx(renameAll: SqlxRename.snakeCase)
final class UserProfile {
  const UserProfile({
    required this.id,
    required this.name,
    required this.address,
    this.bio = '',
    this.sessionActive = false,
    required this.preferences,
    required this.status,
  });

  final int id;

  @Sqlx(rename: 'display_name')
  final String name;

  @Sqlx(flatten: true)
  final Address address;

  final String bio;

  @Sqlx(skip: true)
  final bool sessionActive;

  @Sqlx(json: true)
  final UserPreferences preferences;

  @Sqlx(tryFrom: UserStatusFromInt())
  final UserStatus status;
}

@FromRow()
final class Address {
  const Address({required this.street, required this.city});

  final String street;
  final String city;
}

final class UserPreferences {
  const UserPreferences({required this.darkMode, required this.notifications});

  factory UserPreferences.fromJson(Map<String, Object?> json) {
    return UserPreferences(
      darkMode: json['darkMode'] as bool,
      notifications: json['notifications'] as bool,
    );
  }

  final bool darkMode;
  final bool notifications;
}

enum UserStatus { inactive, active }

final class UserStatusFromInt implements SqlxTryFrom<UserStatus, int> {
  const UserStatusFromInt();

  @override
  UserStatus decode(int value) => switch (value) {
    1 => UserStatus.active,
    0 => UserStatus.inactive,
    _ => throw ArgumentError.value(value, 'value', 'Unknown user status'),
  };
}

@DustDb(driver: Driver.sqflite, migrations: 'migrations')
abstract interface class UserRepository {
  factory UserRepository(dynamic db) = _$UserRepository;

  @Query('SELECT id, display_name, street, city, bio, preferences, status FROM user_profiles WHERE id = ?')
  Future<UserProfile?> findById(int id);

  @Query('SELECT id, display_name, street, city, bio, preferences, status FROM user_profiles')
  Future<List<UserProfile>> listProfiles();

  @Query('SELECT COUNT(*) FROM user_profiles')
  Future<int> countProfiles();

  @Transaction()
  @Query('UPDATE user_profiles SET display_name = ? WHERE id = ?')
  Future<void> renameProfile(String name, int id);
}
