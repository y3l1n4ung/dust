import 'package:dust_db_runtime/dust_db_runtime.dart';
import 'package:dust_db_example/user_repository.dart';
import 'package:test/test.dart';

void main() {
  test('sqlite database maps rows with sqlx-style options', () async {
    final app = AppDatabase.open(':memory:');
    addTearDown(() async {
      await app.pool.close();
    });
    final users = UserDao(app.pool);

    await users.renameProfile('unused', -1);
    await app.pool.transaction((tx) async {
      await UserDao(tx).renameProfile('unused', -1);
      return const Ok<void, SqlxError>(null);
    });

    await app.pool.transaction((tx) async {
      await tx.queryRawInsert();
      return const Ok<void, SqlxError>(null);
    });

    final profile = await unwrap(users.findById(7));

    expect(profile, isNotNull);
    expect(profile!.id, 7);
    expect(profile.name, 'Ada');
    expect(profile.address.street, '42 Compiler Ave');
    expect(profile.address.city, 'Dartmouth');
    expect(profile.bio, '');
    expect(profile.sessionActive, isFalse);
    expect(profile.preferences.darkMode, isTrue);
    expect(profile.preferences.notifications, isFalse);
    expect(profile.status, UserStatus.active);
  });

  test('sqlite database supports scalar and execute queries', () async {
    final app = AppDatabase.open(':memory:');
    addTearDown(() async {
      await app.pool.close();
    });
    final users = UserDao(app.pool);

    await app.pool.queryRawInsert();

    expect(await unwrap(users.countProfiles()), 1);
    final result = await unwrap(users.renameProfile('Grace', 7));
    expect(result.rowsAffected, 1);

    final profile = await unwrap(users.findById(7));
    expect(profile!.name, 'Grace');
  });
}

extension _SeedQueries on SqlxDriver {
  Future<void> queryRawInsert() async {
    await queryExecute(
      r'INSERT INTO user_profiles (id, display_name, street, city, bio, preferences, status) VALUES ($1, $2, $3, $4, $5, $6, $7)',
      [
        7,
        'Ada',
        '42 Compiler Ave',
        'Dartmouth',
        '',
        '{"darkMode":true,"notifications":false}',
        1,
      ],
    ).execute(this);
  }
}

Future<T> unwrap<T>(Future<Result<T, SqlxError>> future) async {
  final result = await future;
  return result.match(
    ok: (value) => value,
    err: (error) => throw StateError('Unexpected SQLx error: $error'),
  );
}
