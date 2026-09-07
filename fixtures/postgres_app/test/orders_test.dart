import 'dart:io';

import 'package:dust_dart/db.dart';
import 'package:postgres_app/postgres_app.dart';
import 'package:test/test.dart';

/// What generated PostgreSQL code does against a real server.
///
/// Skipped unless `DUST_DATABASE_URL` names a database this run may write to.
/// There is no in-memory PostgreSQL, so a fixture cannot bring its own — the
/// same reason `dust db build` needs a server and CI validates from the
/// committed query cache.
void main() {
  final url = Platform.environment['DUST_DATABASE_URL'];
  if (url == null) {
    test('postgres fixture', () {}, skip: 'set DUST_DATABASE_URL to run');
    return;
  }

  late AppDatabase database;
  late OrdersRepo orders;
  late int accountId;

  setUp(() async {
    database = AppDatabase.connect(url);
    orders = OrdersRepo(database.connection);

    // The fixture owns its tables for the length of a test. The migration
    // bookkeeping goes too: leaving it would mark the migration applied and the
    // next test would find no tables.
    await _reset(database);
    _ok(await database.migrate());

    final account = await database.unsafe.fetchAs<int>(
      r'INSERT INTO accounts (email) VALUES ($1) RETURNING id',
      const <Object?>['ada@example.com'],
      (row) => row.read<int>('id'),
    );
    accountId = _ok(account).single;
  });

  tearDown(() async {
    await _reset(database);
    await database.close();
  });

  test('RETURNING reads back the row PostgreSQL generated', () async {
    final placed = _ok(await orders.place(accountId, 'shirt', 2, true));

    expect(placed.id, greaterThan(0));
    expect(placed.item, 'shirt');
    // A real boolean, not the 0 or 1 SQLite stores.
    expect(placed.express, isTrue);
    // A decoded timestamptz, not ISO-8601 text.
    expect(placed.placedAt.isUtc, isTrue);
  });

  test('a list binds as an array for ANY', () async {
    final first = _ok(await orders.place(accountId, 'shirt', 1, false));
    _ok(await orders.place(accountId, 'socks', 1, false));
    final third = _ok(await orders.place(accountId, 'hat', 1, false));

    final found = _ok(await orders.byIds(<int>[first.id, third.id]));

    expect(found.map((order) => order.item), <String>['shirt', 'hat']);
  });

  test('an empty array selects nothing', () async {
    _ok(await orders.place(accountId, 'shirt', 1, false));

    expect(_ok(await orders.byIds(const <int>[])), isEmpty);
  });

  test('a repeated placeholder binds one argument', () async {
    _ok(await orders.place(accountId, 'shirt', 1, false));
    _ok(await orders.place(accountId, 'socks', 5, false));

    // `$2` appears twice in the query and is bound once, where the SQLite
    // driver would bind it twice.
    expect(_ok(await orders.atLeast(accountId, 0)).length, 2);
    expect(_ok(await orders.atLeast(accountId, 3)).length, 1);
  });

  test('a scalar query reads column zero', () async {
    _ok(await orders.place(accountId, 'shirt', 1, false));
    _ok(await orders.place(accountId, 'socks', 1, false));

    expect(_ok(await orders.countFor(accountId)), 2);
  });

  test('a row query returns every column the type declares', () async {
    _ok(await orders.place(accountId, 'shirt', 3, true));

    final all = _ok(await orders.forAccount(accountId));

    expect(all.single.accountId, accountId);
    expect(all.single.quantity, 3);
  });

  test('a violated constraint is a value, not a throw', () async {
    // `quantity > 0` is in the migration, so the database refuses this.
    final result = await orders.place(accountId, 'shirt', 0, false);

    expect(result.isErr, isTrue);
  });
}

/// Drops everything the fixture owns, including the migration bookkeeping.
Future<void> _reset(AppDatabase database) async {
  for (final table in const <String>[
    'orders',
    'accounts',
    '__dust_schema_migrations',
  ]) {
    await database.unsafe.execute('DROP TABLE IF EXISTS $table', const []);
  }
}

/// Returns the `Ok` value, failing the test with the error otherwise.
T _ok<T>(Result<T, SqlxError> result) =>
    result.unwrapOrElse((error) => fail('expected Ok, got $error'));
