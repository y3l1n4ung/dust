import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';
import 'package:test/test.dart';

import 'support/postgres.dart';

/// Every constraint a statement can break, read from the server's SQLSTATE
/// rather than from message text.
void main() {
  if (databaseUrl == null) {
    test('constraint kinds', () {}, skip: skipWithoutDatabase);
    return;
  }

  late PostgresDriver db;

  setUp(() async {
    db = connect();
    await reset(db, <String>['kind_players', 'kind_teams']);
    for (final ddl in const [
      'CREATE TABLE kind_teams (id BIGINT PRIMARY KEY, name TEXT NOT NULL UNIQUE)',
      '''
CREATE TABLE kind_players (
  id      BIGINT PRIMARY KEY,
  team_id BIGINT NOT NULL
    REFERENCES kind_teams (id) DEFERRABLE INITIALLY DEFERRED,
  age     INT CHECK (age >= 0)
)''',
      "INSERT INTO kind_teams (id, name) VALUES (1, 'red')",
    ]) {
      expectOk(await db.unsafe.execute(ddl, const []));
    }
    addTearDown(() async {
      await reset(db, <String>['kind_players', 'kind_teams']);
      await db.close();
    });
  });

  Future<SqlxError> failing(String sql, List<Object?> parameters) async {
    return expectErr(await db.execute(sql, parameters));
  }

  test('a duplicate unique value is a unique violation', () async {
    final error = await failing(
      r'INSERT INTO kind_teams (id, name) VALUES ($1, $2)',
      const <Object?>[2, 'red'],
    );

    expect(error.category, SqlxErrorCategory.query);
    expect(error.kind, SqlxErrorKind.uniqueViolation);
  });

  test('a duplicate primary key is a unique violation', () async {
    final error = await failing(
      r'INSERT INTO kind_teams (id, name) VALUES ($1, $2)',
      const <Object?>[1, 'blue'],
    );

    expect(error.kind, SqlxErrorKind.uniqueViolation);
  });

  test('a null in a NOT NULL column is a not-null violation', () async {
    final error = await failing(
      r'INSERT INTO kind_teams (id, name) VALUES ($1, $2)',
      const <Object?>[2, null],
    );

    expect(error.kind, SqlxErrorKind.notNullViolation);
  });

  test('a value a CHECK rejects is a check violation', () async {
    final error = await failing(
      r'INSERT INTO kind_players (id, team_id, age) VALUES ($1, $2, $3)',
      const <Object?>[1, 1, -1],
    );

    expect(error.kind, SqlxErrorKind.checkViolation);
  });

  test('a deferred foreign key fails at commit as a foreign key violation',
      () async {
    final result = await db.transaction<Unit>(
      (tx) async => (await tx.execute(
        r'INSERT INTO kind_players (id, team_id, age) VALUES ($1, $2, $3)',
        const <Object?>[1, 99, 20],
      ))
          .map((_) => unit),
    );

    final error = expectErr(result);
    expect(error.category, SqlxErrorCategory.transaction);
    expect(error.kind, SqlxErrorKind.foreignKeyViolation);
  });

  test('a failure that breaks no constraint has no kind', () async {
    final error = await failing('SELECT * FROM kind_missing', const []);

    expect(error.category, SqlxErrorCategory.query);
    expect(error.kind, isNull);
  });
}
