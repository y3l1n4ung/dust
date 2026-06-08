# Dust DB

Dust DB is a SQLx-style raw SQL layer for Dart and Flutter. It is not an ORM and does not provide a query builder. App code writes raw SQL in `@Query`, Dust validates it during `dust build --db`, and generated DAO code calls typed `Executor` fetch/execute methods directly.

## Packages

```dart
import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
```

## Database

```dart
@SqlxDatabase(type: SqlxDatabaseType.sqlite)
final class AppDatabase {
  AppDatabase._(this._db);

  final Executor _db;

  late final UserDao users = UserDao(_db);
  late final RawSqlx raw = RawSqlx(_db);

  static Future<AppDatabase> connect(Executor db) async {
    return AppDatabase._(db);
  }

  Future<Result<T, SqlxError>> transaction<T>(
    Future<Result<T, SqlxError>> Function(AppDatabase tx) callback,
  ) {
    return _db.transaction((tx) {
      return callback(AppDatabase._(tx));
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

`Row` is a driver-agnostic interface. Driver packages own concrete adapters such as `Sqlite3Row`, while generated mappers only depend on `Row`. Generated mappers use column-name reads, matching sqlx `FromRow` behavior.

Supported shared reads:

```dart
row.read<int>('id');
row.readNullable<String>('nickname');
row.readBool('active');
row.readBoolNullable('verified');
row.readDateTime('created_at');
row.readDateTimeNullable('deleted_at');
row.readIndex<int>(0); // scalar/raw escape hatches only
```

## DAO Queries

Every DAO uses a redirecting const factory constructor.

```dart
@SqlxDao()
abstract final class UserDao {
  const factory UserDao(Executor db) = _$UserDao;

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

Generated SQLite shape:

```dart
final class _$UserDao implements UserDao {
  const _$UserDao(this._db);

  final Executor _db;

  @override
  Future<Result<UserRow?, SqlxError>> findById(int id) {
    return _db.fetchOptional<UserRow>(
      r'''
  SELECT id, email, name
  FROM users
  WHERE id = ?
  ''',
      [id],
      UserRowFromRow.fromRow,
    );
  }
}
```

## SQLx API Mapping

- `@QueryAs` style returns map to `fetchOptional`, `fetchAll`, or `fetchOne` based on Dart return type.
- `@QueryScalar` style returns map to `fetchScalar` and read column index zero.
- `@Query` statements map to `execute` and return `ExecResult`.
- `@Derive([FromRow()])` generates a static `RowMapper<T>` reference: `UserRowFromRow.fromRow`.

Placeholder rules:

- `@Query` SQL uses SQLx placeholders such as `$1` and `$2`.
- `dust build --db` validates `@Query` SQL with Rust SQLx.
- Generated SQLite DAO code emits SQLite placeholders.

```dart
@Query(r'SELECT id FROM users WHERE id = $1 OR owner_id = $1')
Future<Result<List<UserRow>, SqlxError>> byIdOrOwner(int id);
```

SQLite generated call:

```dart
return _db.fetchAll<UserRow>(
  r'''SELECT id FROM users WHERE id = ? OR owner_id = ?''',
  [id, id],
  UserRowFromRow.fromRow,
);
```

Future Postgres generated DAO code keeps SQLx placeholders when that driver is enabled.

## Native SQLite Access

Use checked DAOs by default. For advanced SQLite operations, cast to `Sqlite3Executor` and use the native `package:sqlite3` database directly.

```dart
final sqlite = (app.pool as Sqlite3Executor).database;
final version = sqlite.select('SELECT sqlite_version()').single[0];
```

Native access also works inside transactions:

```dart
await app.pool.transaction((tx) async {
  final sqlite = (tx as Sqlite3Executor).database;
  sqlite.execute('PRAGMA foreign_keys = ON');
  return const Ok(unit);
});
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

Raw SQL is driver-native because Dust does not validate or rewrite it:

```dart
// SQLite raw SQL uses `?`.
await db.raw.fetch('SELECT * FROM users WHERE id = ?', [id]);

// Future Postgres raw SQL keeps `$1`.
await postgres.raw.fetch(r'SELECT * FROM users WHERE id = $1', [id]);
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

SQLite migrations are applied in sorted file-name order and recorded in `__dust_schema_migrations`, so reopening a Flutter app database skips already applied migrations and applies only new upgrade files.

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
