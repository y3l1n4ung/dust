import 'package:dust_db_runtime/dust_db_runtime.dart';
import 'package:test/test.dart';

final class UserRow {
  const UserRow(this.id);

  final int id;
}

final class FakeRow implements Row {
  FakeRow(this.values, this.columns);

  final List<Object?> values;
  final Map<String, Object?> columns;

  @override
  T read<T>(String column) => columns[column] as T;

  @override
  T? readOrNull<T>(String column) => columns[column] as T?;

  @override
  T readIndex<T>(int index) => values[index] as T;

  @override
  T? readIndexOrNull<T>(int index) => values[index] as T?;

  @override
  bool readBool(String column) => read<Object?>(column) == 1 || read<Object?>(column) == true;

  @override
  bool? readBoolOrNull(String column) {
    final value = readOrNull<Object?>(column);
    return value == null ? null : value == 1 || value == true;
  }

  @override
  DateTime readDateTime(String column) => DateTime.parse(read<String>(column));

  @override
  DateTime? readDateTimeOrNull(String column) {
    final value = readOrNull<String>(column);
    return value == null ? null : DateTime.parse(value);
  }
}

final class FakePool implements Pool {
  FakePool(this.rows);

  final List<Row> rows;
  String? lastSql;
  List<Object?>? lastParameters;

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
    lastSql = sql;
    lastParameters = parameters;
    if (rows.isEmpty) return Ok<T?, SqlxError>(null);
    return Ok<T?, SqlxError>(mapper(rows.first));
  }

  @override
  Future<Result<List<T>, SqlxError>> fetchAll<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    lastSql = sql;
    lastParameters = parameters;
    return Ok<List<T>, SqlxError>([
      for (final row in rows) mapper(row),
    ]);
  }

  @override
  Future<Result<T, SqlxError>> fetchOne<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    lastSql = sql;
    lastParameters = parameters;
    return Ok<T, SqlxError>(mapper(rows.single));
  }

  @override
  Future<Result<T, SqlxError>> fetchScalar<T>(
    String sql,
    List<Object?> parameters,
  ) async {
    lastSql = sql;
    lastParameters = parameters;
    return Ok<T, SqlxError>(rows.single.readIndex<T>(0));
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(String sql, List<Object?> parameters) async {
    lastSql = sql;
    lastParameters = parameters;
    return const Ok<ExecResult, SqlxError>(ExecResult(rowsAffected: 2));
  }

  @override
  Future<Result<T, SqlxError>> transaction<T>(
    Future<Result<T, SqlxError>> Function(SqlxDriver tx) fn,
  ) {
    throw UnimplementedError();
  }

  @override
  Future<Result<Unit, SqlxError>> close() async => const Ok<Unit, SqlxError>(unit);
}

final class _FakeRawSql implements RawSql {
  const _FakeRawSql(this._pool);

  final FakePool _pool;

  @override
  Future<Result<List<Row>, SqlxError>> fetch(String sql, List<Object?> parameters) async {
    _pool.lastSql = sql;
    _pool.lastParameters = parameters;
    return Ok<List<Row>, SqlxError>(_pool.rows);
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  ) {
    return _pool.execute(sql, parameters);
  }
}

void main() {
  tearDown(RowMapperRegistry.resetForTest);

  test('result andThen chains ok and preserves err', () {
    final ok = const Ok<int, SqlxError>(2).andThen<String>(
      (value) => Ok<String, SqlxError>('value:$value'),
    );
    final err = const Err<int, SqlxError>(
      SqlxDecodeError('bad'),
    ).andThen<String>((value) => Ok<String, SqlxError>('value:$value'));

    expect(ok, isA<Ok<String, SqlxError>>());
    expect(ok.match(ok: (value) => value, err: (_) => 'err'), 'value:2');
    expect(err, isA<Err<String, SqlxError>>());
  });

  test('sqlx error factories preserve details', () {
    final driver = SqlxError.driver('driver failed', cause: 'cause');
    final decode = SqlxError.decode('decode failed', cause: 'cause');
    final cardinality = SqlxError.tooManyRows(expected: 1, actual: 3);

    expect(driver, isA<SqlxDriverError>());
    expect(decode, isA<SqlxDecodeError>());
    expect(cardinality, isA<SqlxCardinalityError>());
    expect((cardinality as SqlxCardinalityError).actual, 3);
  });

  test('queryAs maps through generated registry', () async {
    registerRowMapper<UserRow>((row) => UserRow(row.read<int>('id')));
    final pool = FakePool([
      FakeRow([7], {'id': 7}),
    ]);

    final row = await queryAs<UserRow>('SELECT id FROM users', []).fetchOne(pool);

    expect(row.id, 7);
  });

  test('queryScalar reads first selected column', () async {
    final pool = FakePool([
      FakeRow([3], {'count': 3}),
    ]);

    final count = await queryScalar<int>('SELECT COUNT(*) FROM users', []).fetchOne(pool);

    expect(count, 3);
  });

  test('queryExecute returns affected row count', () async {
    final pool = FakePool([]);

    final result = await queryExecute(r'UPDATE users SET name = $1', ['Ada']).execute(pool);

    expect(result.rowsAffected, 2);
  });

  test('sqlite placeholder rewrite duplicates repeated params and ignores SQL literals', () {
    final prepared = rewriteOrdinalPlaceholdersForSqlite(
      r"SELECT '$1' AS label WHERE id = $1 OR owner_id = $1 AND name = $2",
      [7, 'Ada'],
    );

    expect(
      prepared.sql,
      r"SELECT '$1' AS label WHERE id = ? OR owner_id = ? AND name = ?",
    );
    expect(prepared.parameters, [7, 7, 'Ada']);
  });
}
