# Dust DB

Dust DB is a SQLx-style raw SQL layer for Dart and Flutter. It is not an ORM and does not provide a query builder. App code writes raw SQL in `@Query`, Dust validates it during `dust build --db`, and generated DAO code calls `SqlxDriver.fetch` or `SqlxDriver.execute` directly.

## Packages

```dart
import 'package:dust_db_annotation/dust_db_annotation.dart';
import 'package:dust_db_runtime/dust_db_runtime.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
```

## Database

```dart
@SqlxDatabase(type: SqlxDatabaseType.postgres)
final class AppDatabase {
  AppDatabase._(this._db);

  final SqlxDriver _db;

  late final UserDao users = UserDao(_db);
  late final RawSqlx raw = RawSqlx(_db);

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

## Row Mapping

```dart
part 'user_row.g.dart';

@Derive([FromRow()])
@Sqlx(renameAll: SqlxRename.snakeCase)
final class UserRow {
  const UserRow({
    required this.id,
    required this.email,
    required this.name,
  });

  final int id;
  final String email;
  final String name;
}
```

Generated mapper shape:

```dart
extension UserRowFromRow on UserRow {
  static UserRow fromRow(Row row) {
    return UserRow(
      id: row.read<int>('id'),
      email: row.read<String>('email'),
      name: row.read<String>('name'),
    );
  }
}
```

## DAO Queries

Every DAO uses a redirecting const factory constructor.

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

  @Query(r'SELECT COUNT(*) FROM users')
  Future<Result<int, SqlxError>> count();

  @Query(r'INSERT INTO users (email) VALUES ($1)')
  Future<Result<ExecResult, SqlxError>> create(String email);
}
```

Generated shape:

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

## Complex SQL

Simple and complex checked queries both use `@Query(raw SQL)`.

```dart
@Query(r'''
WITH recent_orders AS (
  SELECT user_id, COUNT(*) AS order_count
  FROM orders
  WHERE created_at >= $1
  GROUP BY user_id
)
SELECT
  u.id,
  u.email,
  ro.order_count
FROM users u
LEFT JOIN recent_orders ro ON ro.user_id = u.id
WHERE u.active = true
ORDER BY ro.order_count DESC
LIMIT $2 OFFSET $3
''')
Future<Result<List<UserStatsRow>, SqlxError>> topUsers(
  DateTime since,
  int limit,
  int offset,
);
```

No query builder. No ORM filters. Use the database engine SQL directly.

## Dynamic SQL

Dynamic/admin SQL is explicit and unchecked:

```dart
final result = await db.raw.fetch(
  'SELECT * FROM $tableName WHERE id = ?',
  [id],
);
```

Final rule:

```text
Simple query       -> @Query(raw SQL) checked by dust build --db
Complex query      -> @Query(raw SQL) checked by dust build --db
Dynamic/admin SQL  -> db.raw.fetch(...) runtime only, unchecked
```

## Validation

Run DB validation and DB generation with:

```sh
dust build --db
```

Normal `dust build` does not run SQLx validation.

Dust validates SQL syntax, migrations, table/column existence, placeholder count, result shape, nullability, `FromRow` compatibility, and `Result<T, SqlxError>` return shape.

## Pipeline Split

Dust DB has two separate generation paths.

Normal `dust build` owns DTO/row mapper generation:

- reads `@Derive([FromRow()])`;
- emits `UserRowFromRow.fromRow`;
- emits row mapper registration;
- does not generate `@SqlxDatabase`, `@SqlxDao`, or `@Query` output;
- does not run SQLx validation.

DB mode owns database and DAO generation:

- reads `@SqlxDatabase`;
- reads `@SqlxDao`;
- reads `@Query`;
- validates SQL with SQLx;
- emits database open code and DAO implementations;
- does not emit DTO row mappers.

Keep row DTOs and database/DAO roots in separate Dart libraries when possible:

```text
lib/db/app_database.dart      -> @SqlxDatabase, @SqlxDao, @Query
lib/db/user_row.dart          -> @Derive([FromRow()])
lib/db/user_row.g.dart        -> normal dust build output
lib/db/app_database.g.dart    -> dust build --db output
```

This keeps the normal build and `--db` build from owning the same generated file.
