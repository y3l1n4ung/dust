import 'package:dust_dart/db.dart';
import 'package:test/test.dart';

void main() {
  tearDown(RowMapperRegistry.resetForTest);

  test('DB annotations and value types expose stable configuration', () {
    const database = SqlxDatabase(driver: Driver.postgres);
    const typed = SqlxDatabase(type: SqlxDatabaseType.sqlite);
    const dao = SqlxDao();
    const query = Query(r'SELECT 1');
    const fromRow = FromRow();
    const sqlx = Sqlx(rename: 'display_name', defaultValue: 'anon');
    const tryFrom = _IntStringTryFrom();
    const result = ExecResult(rowsAffected: 2, lastInsertId: 9);

    expect(database.type, SqlxDatabaseType.postgres);
    expect(database.migrations, './migrations');
    expect(typed.type, SqlxDatabaseType.sqlite);
    expect(dao, isA<SqlxDao>());
    expect(query.sql, r'SELECT 1');
    expect(fromRow, isA<FromRow>());
    expect(sqlx.rename, 'display_name');
    expect(sqlx.defaultValue, 'anon');
    expect(tryFrom.decode('9'), 9);
    expect(result.rowsAffected, 2);
    expect(result.lastInsertId, 9);
  });

  test('SqlxError variants expose useful string output', () {
    final driver = SqlxError.driver('driver failed');
    final driverCause = SqlxError.driver('driver failed', cause: 'boom');
    final decode = SqlxError.decode('decode failed');
    final decodeCause = SqlxError.decode('decode failed', cause: 'bad');
    final noRows = SqlxError.noRows('SELECT 1');
    final nullColumn = SqlxError.nullColumn('name');
    final tooMany = SqlxError.tooManyRows(expected: 1, actual: 2);

    expect(driver.toString(), 'driver failed');
    expect(driverCause.toString(), 'driver failed Cause: boom');
    expect(decode.toString(), 'decode failed');
    expect(decodeCause.toString(), 'decode failed Cause: bad');
    expect(noRows.toString(), 'SQL query `SELECT 1` expected 1 row(s), got 0.');
    expect(nullColumn.toString(), 'Column `name` is null.');
    expect(tooMany.toString(), 'SQL query expected 1 row(s), got 2.');
  });

  test('DB JSON helper decodes objects and rejects non-objects', () {
    expect(decodeJsonObject('{"id":1}'), <String, Object?>{'id': 1});
    expect(() => decodeJsonObject('[1]'), throwsA(isA<FormatException>()));
  });

  test('query helpers delegate to Executor fetch methods', () async {
    registerRowMapper<_User>(_UserFromRow.fromRow);
    final executor = _FakeExecutor();

    final one = await queryAs<_User>('one', const []).fetchOne(executor);
    final optional = await queryAs<_User>(
      'optional',
      const [],
    ).fetchOptional(executor);
    final all = await queryAs<_User>('all', const []).fetchAll(executor);
    final scalar = await queryScalar<int>(
      'scalar',
      const [],
    ).fetchOne(executor);
    final nullableScalar = await queryScalar<int>(
      'nullable',
      const [],
    ).fetchOptional(executor);
    final raw = await queryRaw('raw', const []).fetch(executor);
    final exec = await queryExecute('exec', const []).execute(executor);
    final rawx = await RawSqlx(executor).fetch('rawx', const []);
    final rawxExec = await RawSqlx(executor).execute('rawxExec', const []);

    expect(one.id, 1);
    expect(optional?.id, 2);
    expect(all.map((user) => user.id), <int>[3, 4]);
    expect(scalar, 42);
    expect(nullableScalar, isNull);
    expect(raw.single.read<int>('id'), 5);
    expect(exec.rowsAffected, 6);
    expect(
      rawx.match(ok: (rows) => rows.single.read<int>('id'), err: (_) => -1),
      5,
    );
    expect(
      rawxExec.match(ok: (value) => value.rowsAffected, err: (_) => -1),
      1,
    );
    expect(executor.calls, <String>[
      'fetchOne:one',
      'fetchOptional:optional',
      'fetchAll:all',
      'fetchScalar:scalar',
      'fetchScalar:nullable',
      'raw.fetch:raw',
      'execute:exec',
      'raw.fetch:rawx',
      'raw.execute:rawxExec',
    ]);
  });

  test('query helpers throw StateError when Executor returns Err', () async {
    final executor = _FakeExecutor(fail: true);

    expect(
      () => queryExecute('broken', const []).execute(executor),
      throwsA(isA<StateError>()),
    );
  });
}

final class _IntStringTryFrom implements SqlxTryFrom<int, String> {
  const _IntStringTryFrom();

  @override
  int decode(String value) => int.parse(value);
}

final class _User {
  const _User(this.id);

  final int id;
}

extension _UserFromRow on _User {
  static _User fromRow(Row row) => _User(row.read<int>('id'));
}

final class _FakeExecutor implements Executor {
  _FakeExecutor({this.fail = false});

  final bool fail;
  final calls = <String>[];

  @override
  Driver get driver => Driver.sqlite3;

  @override
  RawSql get raw => _FakeRawSql(this);

  @override
  Future<Result<T?, SqlxError>> fetchOptional<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    calls.add('fetchOptional:$sql');
    if (fail) return Err<T?, SqlxError>(SqlxError.driver('failed'));
    return Ok<T?, SqlxError>(mapper(const _StaticRow(2)));
  }

  @override
  Future<Result<List<T>, SqlxError>> fetchAll<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    calls.add('fetchAll:$sql');
    if (fail) return Err<List<T>, SqlxError>(SqlxError.driver('failed'));
    return Ok<List<T>, SqlxError>([
      mapper(const _StaticRow(3)),
      mapper(const _StaticRow(4)),
    ]);
  }

  @override
  Future<Result<T, SqlxError>> fetchOne<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    calls.add('fetchOne:$sql');
    if (fail) return Err<T, SqlxError>(SqlxError.driver('failed'));
    return Ok<T, SqlxError>(mapper(const _StaticRow(1)));
  }

  @override
  Future<Result<T, SqlxError>> fetchScalar<T>(
    String sql,
    List<Object?> parameters,
  ) async {
    calls.add('fetchScalar:$sql');
    if (fail) return Err<T, SqlxError>(SqlxError.driver('failed'));
    final value = sql == 'nullable' ? null : 42;
    return Ok<T, SqlxError>(value as T);
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  ) async {
    calls.add('execute:$sql');
    if (fail) return Err<ExecResult, SqlxError>(SqlxError.driver('failed'));
    return const Ok<ExecResult, SqlxError>(ExecResult(rowsAffected: 6));
  }

  @override
  Future<Result<T, SqlxError>> transaction<T>(
    Future<Result<T, SqlxError>> Function(Executor tx) fn,
  ) {
    return fn(this);
  }

  @override
  Future<Result<Unit, SqlxError>> close() async {
    return const Ok<Unit, SqlxError>(unit);
  }
}

final class _FakeRawSql implements RawSql {
  const _FakeRawSql(this._executor);

  final _FakeExecutor _executor;

  @override
  Future<Result<List<Row>, SqlxError>> fetch(
    String sql,
    List<Object?> parameters,
  ) async {
    _executor.calls.add('raw.fetch:$sql');
    if (_executor.fail) {
      return Err<List<Row>, SqlxError>(SqlxError.driver('failed'));
    }
    return const Ok<List<Row>, SqlxError>([_StaticRow(5)]);
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  ) async {
    _executor.calls.add('raw.execute:$sql');
    if (_executor.fail) {
      return Err<ExecResult, SqlxError>(SqlxError.driver('failed'));
    }
    return const Ok<ExecResult, SqlxError>(ExecResult(rowsAffected: 1));
  }
}

final class _StaticRow implements Row {
  const _StaticRow(this.id);

  final int id;

  @override
  T read<T>(String column) => readNullable<T>(column) as T;

  @override
  T? readNullable<T>(String column) => id as T?;

  @override
  T readIndex<T>(int index) => id as T;

  @override
  T? readIndexNullable<T>(int index) => id as T?;

  @override
  bool readBool(String column) => id != 0;

  @override
  bool? readBoolNullable(String column) => id != 0;

  @override
  DateTime readDateTime(String column) => DateTime.utc(2026);

  @override
  DateTime? readDateTimeNullable(String column) => DateTime.utc(2026);
}
