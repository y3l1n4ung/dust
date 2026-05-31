import 'package:dust_db_annotation/dust_db_annotation.dart';
import 'package:dust_db_runtime/dust_db_runtime.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

part 'user_repository.g.dart';

@Derive([FromRow()])
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

@Derive([FromRow()])
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

@SqlxDatabase(type: SqlxDatabaseType.sqlite, migrations: './migrations')
abstract class AppDatabase {
  factory AppDatabase.open(String path) = _$AppDatabase.open;

  Pool get pool;
}

@SqlxDao()
abstract final class UserDao {
  const factory UserDao(SqlxDriver db) = _$UserDao;

  @Query(r'''
SELECT id, display_name, street, city, bio, preferences, status
FROM user_profiles
WHERE id = $1
''')
  Future<Result<UserProfile?, SqlxError>> findById(int id);

  @Query(r'''
SELECT id, display_name, street, city, bio, preferences, status
FROM user_profiles
''')
  Future<Result<List<UserProfile>, SqlxError>> listProfiles();

  @Query(r'SELECT COUNT(*) FROM user_profiles')
  Future<Result<int, SqlxError>> countProfiles();

  @Query(r'UPDATE user_profiles SET display_name = $1 WHERE id = $2')
  Future<Result<ExecResult, SqlxError>> renameProfile(String name, int id);
}
