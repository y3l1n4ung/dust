import 'package:dust_dart/db.dart';
import 'package:test/test.dart';

void main() {
  test('queryAs decodes with the row deserializer it was given', () async {
    final executor = _CapturingExecutor();

    final user = await queryAs<_User>(
      'SELECT id FROM users',
      const [],
      using: const _UserFromRow(),
    ).fetchOne(executor);

    expect(user.id, 7);
    expect(executor.calls, <String>['fetchOne:SELECT id FROM users']);
  });

  test('queryAs accepts a plain mapper function through the adapter', () async {
    final executor = _CapturingExecutor();

    final user = await queryAs<_User>(
      'SELECT id FROM users',
      const [],
      using: const RowMapperDeserializer<_User>(_User.fromRow),
    ).fetchOne(executor);

    expect(user.id, 7);
  });

  test('withDeserializer replaces the row mapping', () async {
    final executor = _CapturingExecutor();

    final user = await queryAs<_User>(
      'SELECT id FROM users',
      const [],
      using: const _UserFromRow(),
    ).withDeserializer(const _NineUser()).fetchOne(executor);

    expect(user.id, 9);
  });

  test('withMapper replaces the row mapping with a function', () async {
    final executor = _CapturingExecutor();

    final user = await queryAs<_User>(
      'SELECT id FROM users',
      const [],
      using: const _UserFromRow(),
    ).withMapper((_) => const _User(9)).fetchOne(executor);

    expect(user.id, 9);
  });

  test('a row deserializer tears off as a plain mapper', () async {
    final mapper = const _UserFromRow().asMapper;
    final executor = _CapturingExecutor();

    final user = await queryAs<_User>(
      'SELECT id FROM users',
      const [],
      using: RowMapperDeserializer<_User>(mapper),
    ).fetchOne(executor);

    expect(user.id, 7);
  });

  test('the query exposes the mapping it will decode with', () {
    const query = QueryAs<_User>(
      'SELECT id FROM users',
      [],
      using: _UserFromRow(),
    );

    expect(query.rowMapper(const _StaticRow(4)).id, 4);
  });
}

/// Stands in for the `$TypeFromRow` witness Dust generates.
final class _UserFromRow implements RowDeserializer<_User> {
  const _UserFromRow();

  @override
  _User deserialize(Row row) => _User.fromRow(row);
}

final class _NineUser implements RowDeserializer<_User> {
  const _NineUser();

  @override
  _User deserialize(Row row) => const _User(9);
}

final class _User {
  const _User(this.id);

  final int id;

  static _User fromRow(Row row) {
    return _User(row.read<int>('id'));
  }
}

final class _CapturingExecutor implements DatabaseConnection {
  final calls = <String>[];

  @override
  Driver get driver => Driver.sqlite3;

  @override
  Future<Result<T?, SqlxError>> fetchOptional<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) {
    throw UnimplementedError();
  }

  @override
  Future<Result<List<T>, SqlxError>> fetchAll<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) {
    throw UnimplementedError();
  }

  @override
  Future<Result<T, SqlxError>> fetchOne<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    calls.add('fetchOne:$sql');
    return Ok<T, SqlxError>(mapper(const _StaticRow(7)));
  }

  @override
  Future<Result<T, SqlxError>> fetchScalar<T>(
    String sql,
    List<Object?> parameters,
  ) {
    throw UnimplementedError();
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  ) {
    throw UnimplementedError();
  }

  @override
  Future<Result<T, SqlxError>> transaction<T>(
    Future<Result<T, SqlxError>> Function(Executor tx) fn,
  ) {
    throw UnimplementedError();
  }

  @override
  Future<Result<Unit, SqlxError>> close() async {
    return const Ok<Unit, SqlxError>(unit);
  }
}

final class _StaticRow implements Row {
  const _StaticRow(this.id);

  final int id;

  @override
  T read<T>(String column) => id as T;

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
