import 'package:dust_dart/db.dart';

/// Fakes shared by the DB runtime suites.

final class IntStringTryFrom implements SqlxTryFrom<int, String> {
  const IntStringTryFrom();

  @override
  int decode(String value) => int.parse(value);
}

final class User {
  const User(this.id);

  final int id;
}

extension UserFromRow on User {
  static User fromRow(Row row) => User(row.read<int>('id'));
}

final class FakeDatabaseClient implements DatabaseClient {
  const FakeDatabaseClient(this.connection, this.unsafe);

  @override
  final Connection connection;

  @override
  final UnsafeSql unsafe;

  @override
  Future<Result<Unit, SqlxError>> migrate() async =>
      const Ok<Unit, SqlxError>(unit);
}

final class FakeExecutor implements Pool {
  FakeExecutor({this.fail = false});

  final bool fail;
  final calls = <String>[];

  @override
  Driver get driver => Driver.sqlite3;

  @override
  Future<Result<T?, SqlxError>> fetchOptional<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    calls.add('fetchOptional:$sql');
    if (fail) return Err<T?, SqlxError>(SqlxError.driver('failed'));
    return Ok<T?, SqlxError>(mapper(const StaticRow(2)));
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
      mapper(const StaticRow(3)),
      mapper(const StaticRow(4)),
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
    return Ok<T, SqlxError>(mapper(const StaticRow(1)));
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
    Future<Result<T, SqlxError>> Function(Transaction tx) fn,
  ) {
    throw UnimplementedError();
  }

  @override
  Future<Result<Unit, SqlxError>> close() async {
    return const Ok<Unit, SqlxError>(unit);
  }
}

/// Stand-in for what a driver package hands a generated facade.
final class FakeUnsafeSql implements UnsafeSql {
  const FakeUnsafeSql(this._executor);

  final FakeExecutor _executor;

  @override
  Future<Result<List<Row>, SqlxError>> fetch(
    String sql,
    List<Object?> parameters,
  ) async {
    _executor.calls.add('unsafe.fetch:$sql');
    if (_executor.fail) {
      return Err<List<Row>, SqlxError>(SqlxError.driver('failed'));
    }
    return const Ok<List<Row>, SqlxError>([StaticRow(5)]);
  }

  @override
  Future<Result<List<T>, SqlxError>> fetchAs<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    final rows = await fetch(sql, parameters);
    return rows.map((rows) => <T>[for (final row in rows) mapper(row)]);
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  ) async {
    _executor.calls.add('unsafe.execute:$sql');
    if (_executor.fail) {
      return Err<ExecResult, SqlxError>(SqlxError.driver('failed'));
    }
    return const Ok<ExecResult, SqlxError>(ExecResult(rowsAffected: 1));
  }
}

final class StaticRow implements Row {
  const StaticRow(this.id);

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
