import 'package:dust_dart/db.dart';
import 'package:test/test.dart';

import 'db_runtime_support.dart';

/// The query helpers, and how a facade delegates to its connection.
void main() {
  test('query helpers delegate to Executor fetch methods', () async {
    const mapper = UserFromRow.fromRow;
    final executor = FakeExecutor();
    final client = FakeDatabaseClient(executor, FakeUnsafeSql(executor));

    final one = await queryAs<User>(
      'one',
      const [],
    ).fetchOneWith(executor, mapper);
    final optional = await queryAs<User>(
      'optional',
      const [],
    ).fetchOptionalWith(executor, mapper);
    final all = await queryAs<User>(
      'all',
      const [],
    ).fetchAllWith(executor, mapper);
    final scalar = await queryScalar<int>(
      'scalar',
      const [],
    ).fetchOne(executor);
    final nullableScalar = await queryScalar<int>(
      'nullable',
      const [],
    ).fetchOptional(executor);
    final exec = await queryExecute('exec', const []).execute(executor);
    final unsafeRows = await client.unsafe.fetch('rawx', const []);
    final unsafeExec = await client.unsafe.execute('rawxExec', const []);

    expect(executor, isA<Executor>());
    expect(executor, isA<Connection>());
    expect(client.executor, same(executor));
    expect(one.match(ok: (user) => user.id, err: (_) => -1), 1);
    expect(optional.match(ok: (user) => user?.id, err: (_) => -1), 2);
    expect(
      all.match(ok: (users) => users.map((user) => user.id), err: (_) => null),
      <int>[3, 4],
    );
    expect(scalar.match(ok: (value) => value, err: (_) => -1), 42);
    expect(nullableScalar.match(ok: (value) => value, err: (_) => -1), isNull);

    expect(exec.match(ok: (value) => value.rowsAffected, err: (_) => -1), 6);
    expect(
      unsafeRows.match(
          ok: (rows) => rows.single.read<int>('id'), err: (_) => -1),
      5,
    );
    expect(
      unsafeExec.match(ok: (value) => value.rowsAffected, err: (_) => -1),
      1,
    );
    expect(executor.calls, <String>[
      'fetchOne:one',
      'fetchOptional:optional',
      'fetchAll:all',
      'fetchScalar:scalar',
      'fetchScalar:nullable',
      'execute:exec',
      'unsafe.fetch:rawx',
      'unsafe.execute:rawxExec',
    ]);
  });

  test('the facade delegates transactions and closing to its connection',
      () async {
    // `DatabaseClientExecution` is what an application calls; the connection
    // underneath is what actually does the work.
    final executor = FakeExecutor();
    final client = FakeDatabaseClient(executor, FakeUnsafeSql(executor));

    expect(client.executor, same(executor));
    expect((await client.close()).isOk, isTrue);
    expect(
      () => client.transaction<Unit>(
        (tx) async => const Ok<Unit, SqlxError>(unit),
      ),
      throwsA(isA<UnimplementedError>()),
    );
  });

  test('query helpers hand back the executor Err rather than throwing',
      () async {
    final executor = FakeExecutor(fail: true);

    final result = await queryExecute('broken', const []).execute(executor);

    expect(result.isErr, isTrue);
    expect(
        result.match(ok: (_) => null, err: (error) => error), isA<SqlxError>());
  });
}
