# Dust DB Specification

## Final API

Dust DB is SQL-first and SQLx-style:

- `@SqlxDatabase` marks the top-level database type.
- `@SqlxDao` marks generated DAO interfaces.
- `@Query` contains raw SQL and is checked only by `dust build --db`.
- DAO methods return `Future<Result<T, SqlxError>>`.
- Generated DAO classes use typed `SqlxDriver` fetch/execute methods.
- Dynamic SQL uses `RawSqlx`/`db.raw` and is runtime-only unchecked.

No ORM. No query builder. No cross-dialect SQL abstraction.

## Runtime API

```dart
abstract interface class SqlxDriver {
  Driver get driver;
  RawSql get raw;

  Future<Result<List<Row>, SqlxError>> fetch(
    String sql,
    List<Object?> parameters,
  );

  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  );

  Future<Result<T, SqlxError>> transaction<T>(
    Future<Result<T, SqlxError>> Function(SqlxDriver tx) fn,
  );

  Future<Result<Unit, SqlxError>> close();
}
```

## DAO Shape

```dart
@SqlxDao()
abstract final class UserDao {
  const factory UserDao(SqlxDriver db) = _$UserDao;

  @Query(r'''
  SELECT id, email, name
  FROM users
  WHERE id = $1
  ''')
  Future<Result<UserRow?, SqlxError>> findById(int id);
}
```

Generated:

```dart
final class _$UserDao implements UserDao {
  const _$UserDao(this._db);

  final SqlxDriver _db;

  @override
  Future<Result<UserRow?, SqlxError>> findById(int id) async {
    final result = await _db.fetch(
      r'''
  SELECT id, email, name
  FROM users
  WHERE id = $1
  ''',
      [id],
    );

    return result.andThen((rows) {
      if (rows.isEmpty) return const Ok<UserRow?, SqlxError>(null);
      if (rows.length > 1) {
        return Err<UserRow?, SqlxError>(
          SqlxError.tooManyRows(expected: 1, actual: rows.length),
        );
      }
      try {
        return Ok<UserRow?, SqlxError>(UserRowFromRow.fromRow(rows.first));
      } catch (error) {
        return Err<UserRow?, SqlxError>(
          SqlxError.decode(error.toString(), cause: error),
        );
      }
    });
  }
}
```

## Database Shape

```dart
@SqlxDatabase(type: SqlxDatabaseType.postgres)
final class AppDatabase {
  final SqlxDriver _db;

  late final UserDao users = UserDao(_db);
  late final ProductDao products = ProductDao(_db);
  late final RawSqlx raw = RawSqlx(_db);

  AppDatabase._(this._db);

  static Future<AppDatabase> connect(SqlxDriver driver) async {
    return AppDatabase._(driver);
  }

  Future<Result<T, SqlxError>> transaction<T>(
    Future<Result<T, SqlxError>> Function(AppDatabase tx) callback,
  ) {
    return _db.transaction((txDriver) {
      return callback(AppDatabase._(txDriver));
    });
  }

  Future<Result<Unit, SqlxError>> close() {
    return _db.close();
  }
}
```

## Query Rule

```text
Simple query       -> @Query(raw SQL)
Complex query      -> @Query(raw SQL)
Dynamic/admin SQL  -> db.raw.fetch(...)
```

`@Query` is raw SQL plus build-time SQLx validation.

`db.raw` is raw SQL plus runtime execution only, unchecked.

## SQL Rules

`@Query` SQL uses SQLx placeholders: `$1`, `$2`, `$3`.

Generated DAO SQL uses target driver placeholders:

- SQLite emits `?` placeholders and expands reordered/repeated parameters.
- Future Postgres output keeps `$1`, `$2`, `$3`.

Runtime raw SQL is not rewritten. `db.raw` callers must use the selected
driver's native placeholder form.

Allowed:

```dart
@Query(r'SELECT id, email FROM users WHERE id = $1')
@Query(r'''
SELECT id, email
FROM users
WHERE id = $1
''')
```

Rejected for checked SQL:

```dart
@Query(sqlVariable)
@Query('SELECT * FROM ' + tableName)
@Query('SELECT * FROM users WHERE id = $id')
@Query('SELECT * FROM users WHERE id = ?1')
```

## Validation

Only this command runs SQLx validation:

```sh
dust build --db
```

Normal `dust build` does not validate DB SQL.

`dust build --db` validates:

- `@SqlxDatabase`;
- migrations;
- `@SqlxDao`;
- `@Query` SQL syntax with SQLx;
- table and column existence;
- placeholder count;
- result column names and nullability;
- `FromRow` compatibility;
- `Result<T, SqlxError>` return shape.

## Pipeline Ownership

Dust DB deliberately keeps DTO generation and DB generation separate.

Normal build:

- owns `@Derive([FromRow()])`;
- emits `FromRow` mapper extensions and mapper registration;
- does not claim `@SqlxDatabase`, `@SqlxDao`, or `@Query`;
- does not run SQLx validation.

DB build:

- owns `@SqlxDatabase`, `@SqlxDao`, and `@Query`;
- validates SQL with SQLx;
- emits database open helpers, embedded migrations, and DAO implementations;
- does not emit DTO row mappers.

Recommended layout:

```text
lib/db/app_database.dart      # @SqlxDatabase, @SqlxDao, @Query
lib/db/user_row.dart          # @Derive([FromRow()])
lib/db/app_database.g.dart    # dust build --db output
lib/db/user_row.g.dart        # normal dust build output
```

Generated outputs are file scoped, so mixed DTO/database libraries are allowed for validation but not recommended for clean pipeline ownership.

## Test Plan

- Parse `@SqlxDatabase(type: ...)`.
- Parse `@SqlxDao` redirecting const factory constructors.
- Parse `Future<Result<T, SqlxError>>` return shapes.
- Reject DAO methods without `Result`.
- Reject dynamic SQL inside `@Query`.
- Snapshot generated DAO implementation.
- Snapshot generated database wrapper.
- Validate complex CTE SQL through SQLx.
- Validate dialect-specific SQL against the configured database type.
- Ensure `db.raw` is not SQLx-validated.
