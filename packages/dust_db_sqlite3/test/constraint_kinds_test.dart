import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:test/test.dart';

/// Every constraint a statement can break, read from SQLite's extended result
/// code rather than from message text.
void main() {
  late Sqlite3Driver db;

  setUp(() {
    db = Sqlite3Driver.connect(
      const SqliteConnectOptions.memory(foreignKeys: true),
      migrations: const {
        '0001.sql': '''
CREATE TABLE teams (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE);
CREATE TABLE players (
  id INTEGER PRIMARY KEY,
  team_id INTEGER NOT NULL
    REFERENCES teams (id) DEFERRABLE INITIALLY DEFERRED,
  age INTEGER CHECK (age >= 0)
);
INSERT INTO teams (id, name) VALUES (1, 'red');
''',
      },
    );
  });

  tearDown(() => db.close());

  Future<SqlxError> failing(String sql, List<Object?> parameters) async {
    final result = await db.execute(sql, parameters);
    return switch (result) {
      Ok() => fail('expected `$sql` to fail'),
      Err(:final error) => error,
    };
  }

  test('a duplicate unique value is a unique violation', () async {
    final error = await failing(
      'INSERT INTO teams (id, name) VALUES (?, ?)',
      const [2, 'red'],
    );

    expect(error.category, SqlxErrorCategory.query);
    expect(error.kind, SqlxErrorKind.uniqueViolation);
  });

  test('a duplicate primary key is a unique violation', () async {
    final error = await failing(
      'INSERT INTO teams (id, name) VALUES (?, ?)',
      const [1, 'blue'],
    );

    expect(error.kind, SqlxErrorKind.uniqueViolation);
  });

  test('a null in a NOT NULL column is a not-null violation', () async {
    final error = await failing(
      'INSERT INTO teams (id, name) VALUES (?, ?)',
      const [2, null],
    );

    expect(error.kind, SqlxErrorKind.notNullViolation);
  });

  test('a value a CHECK rejects is a check violation', () async {
    final error = await failing(
      'INSERT INTO players (id, team_id, age) VALUES (?, ?, ?)',
      const [1, 1, -1],
    );

    expect(error.kind, SqlxErrorKind.checkViolation);
  });

  test('a deferred foreign key fails at commit as a foreign key violation',
      () async {
    final result = await db.transaction<Unit>(
      (tx) async => (await tx.execute(
        'INSERT INTO players (id, team_id, age) VALUES (?, ?, ?)',
        const [1, 99, 20],
      ))
          .map((_) => unit),
    );

    final error = switch (result) {
      Ok() => fail('expected the commit to fail'),
      Err(:final error) => error,
    };
    expect(error.category, SqlxErrorCategory.transaction);
    expect(error.kind, SqlxErrorKind.foreignKeyViolation);
  });

  test('a failure that breaks no constraint has no kind', () async {
    final error = await failing('SELECT * FROM missing', const []);

    expect(error.category, SqlxErrorCategory.query);
    expect(error.kind, isNull);
  });
}
