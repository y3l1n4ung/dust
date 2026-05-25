import 'package:dust_db_example/user_repository.dart';
import 'package:test/test.dart';

void main() {
  test('generated repository maps rows with sqlx-style options', () async {
    final db = _FakeDb({
      'id': 7,
      'display_name': 'Ada',
      'street': '42 Compiler Ave',
      'city': 'Dartmouth',
      'preferences': '{"darkMode":true,"notifications":false}',
      'status': 1,
    });
    final repository = UserRepository(db);

    final profile = await repository.findById(7);

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

  test('generated repository supports scalar and transaction methods', () async {
    final db = _FakeDb({'COUNT(*)': 3});
    final repository = UserRepository(db);

    expect(await repository.countProfiles(), 3);
    await repository.renameProfile('Grace', 7);

    expect(db.transactionCount, 1);
    expect(db.executedSql.single, contains('UPDATE user_profiles'));
  });
}

final class _FakeDb {
  _FakeDb(this.row);

  final Map<String, Object?> row;
  final executedSql = <String>[];
  int transactionCount = 0;

  Future<List<Map<String, Object?>>> rawQuery(
    String sql, [
    List<Object?> args = const <Object?>[],
  ]) async {
    return <Map<String, Object?>>[row];
  }

  Future<void> execute(
    String sql, [
    List<Object?> args = const <Object?>[],
  ]) async {
    executedSql.add(sql);
  }

  Future<T> transaction<T>(Future<T> Function(_FakeDb txn) action) async {
    transactionCount += 1;
    return action(this);
  }
}
