import 'package:dust_dart/db.dart';
import 'package:test/test.dart';

void main() {
  test('a generated terminal decodes with the row type own mapping', () async {
    final executor = _CapturingExecutor();

    final user = await queryAs<FakeUser>('SELECT id FROM users', const [])
        .fetchOne(executor);

    expect(user.id, 7);
    expect(executor.calls, <String>['fetchOne:SELECT id FROM users']);
  });

  test('the With terminals take a mapping for a row type Dust does not own',
      () async {
    final executor = _CapturingExecutor();

    final user = await queryAs<FakeUntyped>(
      'SELECT id FROM users',
      const [],
    ).fetchOneWith(executor, (row) => FakeUntyped(row.read<int>('id')));

    expect(user.id, 7);
  });

  test('a row deserializer wraps a plain mapper function', () {
    const deserializer = RowMapperDeserializer<FakeUser>(_$UserFromRow);

    expect(deserializer.deserialize(const _StaticRow(4)).id, 4);
  });

  test('a row deserializer tears off as a plain mapper', () {
    final mapper = const $UserRowDeserializer().asMapper;

    expect(mapper(const _StaticRow(5)).id, 5);
  });
}

// Hand-written stand-ins for what `@Derive([FromRow()])` generates, so this
// package can test the shape without depending on the generator.

FakeUser _$UserFromRow(Row row) => FakeUser(row.read<int>('id'));

final class $UserRowDeserializer implements RowDeserializer<FakeUser> {
  const $UserRowDeserializer();

  @override
  FakeUser deserialize(Row row) => _$UserFromRow(row);
}

extension $UserQuery on QueryAs<FakeUser> {
  Future<FakeUser> fetchOne(DatabaseExecutor db) =>
      fetchOneWith(db, _$UserFromRow);

  Future<FakeUser?> fetchOptional(DatabaseExecutor db) =>
      fetchOptionalWith(db, _$UserFromRow);

  Future<List<FakeUser>> fetchAll(DatabaseExecutor db) =>
      fetchAllWith(db, _$UserFromRow);
}

final class FakeUser {
  const FakeUser(this.id);

  final int id;
}

/// A row type with no generated mapping, so it has no terminals of its own.
final class FakeUntyped {
  const FakeUntyped(this.id);

  final int id;
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
