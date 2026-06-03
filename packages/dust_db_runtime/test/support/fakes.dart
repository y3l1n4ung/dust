import 'package:dust_dart/db.dart';

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
  bool readBool(String column) {
    return read<Object?>(column) == 1 || read<Object?>(column) == true;
  }

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
  FakePool(this.rows, {this.error});

  final List<Row> rows;
  final SqlxError? error;
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
    final error = this.error;
    if (error != null) return Err<T?, SqlxError>(error);
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
    final error = this.error;
    if (error != null) return Err<List<T>, SqlxError>(error);
    return Ok<List<T>, SqlxError>([for (final row in rows) mapper(row)]);
  }

  @override
  Future<Result<T, SqlxError>> fetchOne<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    lastSql = sql;
    lastParameters = parameters;
    final error = this.error;
    if (error != null) return Err<T, SqlxError>(error);
    return Ok<T, SqlxError>(mapper(rows.single));
  }

  @override
  Future<Result<T, SqlxError>> fetchScalar<T>(
    String sql,
    List<Object?> parameters,
  ) async {
    lastSql = sql;
    lastParameters = parameters;
    final error = this.error;
    if (error != null) return Err<T, SqlxError>(error);
    return Ok<T, SqlxError>(rows.single.readIndex<T>(0));
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  ) async {
    lastSql = sql;
    lastParameters = parameters;
    final error = this.error;
    if (error != null) return Err<ExecResult, SqlxError>(error);
    return const Ok<ExecResult, SqlxError>(ExecResult(rowsAffected: 2));
  }

  @override
  Future<Result<T, SqlxError>> transaction<T>(
    Future<Result<T, SqlxError>> Function(SqlxDriver tx) fn,
  ) {
    throw UnimplementedError();
  }

  @override
  Future<Result<Unit, SqlxError>> close() async {
    return const Ok<Unit, SqlxError>(unit);
  }
}

final class _FakeRawSql implements RawSql {
  const _FakeRawSql(this._pool);

  final FakePool _pool;

  @override
  Future<Result<List<Row>, SqlxError>> fetch(
    String sql,
    List<Object?> parameters,
  ) async {
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
