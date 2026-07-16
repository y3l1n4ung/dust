# Database

Dust Database generates typed SQLite access from raw SQL and validates static
queries against your migrations with Rust SQLx. It is not an ORM or query
builder.

Database is currently beta. SQLite through `package:sqlite3` is the supported
runtime; PostgreSQL annotations are reserved for future support.

## Add the Packages

Install the Dust CLI from the [root guide](../../README.md#installation), then
add the annotations and SQLite runtime:

```bash
dart pub add dust_dart dust_db_sqlite3
```

The SQLite package uses native libraries through Dart FFI and is intended for
native Dart and Flutter targets, not web.

## Quick Start

Create a forward-only migration at `migrations/0001_create_users.sql`:

```sql
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL
);
```

Define row models in their own Dart library:

```dart
import 'package:dust_dart/db.dart';

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

Define the database and DAO in `app_database.dart`:

```dart
import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

import 'user_row.dart';

part 'app_database.g.dart';

@SqlxDatabase(
  type: SqlxDatabaseType.sqlite,
  migrations: './migrations',
)
abstract class AppDatabase {
  factory AppDatabase.open(String path) = _$AppDatabase.open;

  Pool get pool;
}

@SqlxDao()
abstract final class UserDao {
  const factory UserDao(Executor db) = _$UserDao;

  @Query(r'SELECT id, email, name FROM users WHERE id = $1')
  Future<Result<UserRow?, SqlxError>> findById(int id);

  @Query(r'SELECT id, email, name FROM users ORDER BY name')
  Future<Result<List<UserRow>, SqlxError>> listUsers();

  @Query(r'SELECT COUNT(*) FROM users')
  Future<Result<int, SqlxError>> countUsers();

  @Query(r'INSERT INTO users (email, name) VALUES ($1, $2)')
  Future<Result<ExecResult, SqlxError>> createUser(
    String email,
    String name,
  );
}
```

Generate row mappers, then validate SQL and generate the database code:

```bash
dust build
dust db build
```

Open the database and pass its pool to a DAO:

```dart
final database = AppDatabase.open('app.db');
final users = UserDao(database.pool);

final result = await users.findById(42);
result.match(
  ok: (user) => print(user?.name),
  err: (error) => print('Database error: $error'),
);

await database.pool.close();
```

> [!IMPORTANT]
> Keep row models and database roots in separate libraries. `dust build` owns
> normal derives such as `FromRow`; `dust db build` owns `@SqlxDatabase`,
> `@SqlxDao`, and `@Query` output.

## Migrations

Dust reads every `.sql` file in the configured directory in sorted filename
order. DB generation embeds those files into the generated opener. At runtime,
SQLite records applied filenames in `__dust_schema_migrations` and runs only
new files.

Use names that sort in application order:

```text
migrations/
  0001_create_users.sql
  0002_add_user_avatar.sql
```

> [!IMPORTANT]
> Do not edit or rename a migration after it ships. Existing installations have
> already recorded its filename; add a new migration for every schema change.

## DAO Return Types

The success type inside `Future<Result<T, SqlxError>>` selects the generated
executor call:

| Success type | Generated behavior |
| :--- | :--- |
| `RowType` | Requires exactly one row and maps it with `FromRow`. |
| `RowType?` | Returns zero or one mapped row. |
| `List<RowType>` | Maps every returned row. |
| `String`, `int`, `double`, `num`, `bool`, or `DateTime` | Reads one scalar from column zero. |
| `List<Row>` | Returns raw driver-agnostic rows. |
| `ExecResult` | Executes the statement and returns affected rows and last insert ID. |
| `Unit` | Executes the statement and discards execution metadata. |

DAO query methods must be abstract, return the exact `Future<Result<...,
SqlxError>>` shape, and use required positional parameters.

## SQL Placeholders

Write checked `@Query` SQL with SQLx-style placeholders:

```dart
@Query(r'SELECT id, email, name FROM users WHERE id = $1 OR owner_id = $1')
Future<Result<List<UserRow>, SqlxError>> byIdOrOwner(int id);
```

For SQLite, Dust rewrites `$1` to `?` and repeats the matching Dart argument
when a placeholder appears more than once. Placeholder numbers must match the
DAO method's positional arguments.

Both simple and complex static SQL belong in `@Query`; CTEs, joins, grouping,
ordering, limits, and offsets are passed to SQLite and SQLx as written.

## Row Mapping

`@Derive([FromRow()])` generates a `TypeFromRow.fromRow(Row row)` mapper. The
mapper reads columns by name through the driver-independent `Row` interface.

`@Sqlx` supports these mapping options:

| Option | Behavior |
| :--- | :--- |
| `renameAll` | Applies a naming rule to every field in the row class. |
| `rename` | Maps one field to an explicit column name. |
| `flatten` | Builds a nested `FromRow` type from the same row. |
| `skip` | Ignores a field; the field or annotation must supply a default. |
| `defaultValue` | Uses a value when the selected column is null. |
| `json` | Decodes a text column through `Type.fromJson(...)`. |
| `tryFrom` | Decodes a database value with a `SqlxTryFrom` converter. |

Directly supported field types are `String`, `int`, `double`, `num`, `bool`,
`DateTime`, and nullable variants. Use `json` or `tryFrom` for custom values.

## Validation

`dust db build` applies migrations to an in-memory SQLite database by default,
asks SQLx to describe each static query, writes generated Dart, and caches query
metadata at:

```text
.dart_tool/dust/db_query_cache_v2.json
```

It validates migration SQL, placeholders, static query syntax, scalar column
count, known row columns, supported return types, and DAO method shapes.

Use a no-write check in CI:

```bash
dust check --db
```

Set `DUST_DATABASE_URL` only when validation must use another SQLite database.

> [!WARNING]
> DB validation applies the configured migrations to `DUST_DATABASE_URL`. Never
> point it at a production or user database.

## Offline Validation

After a successful online `dust db build`, validation can reuse matching cached
metadata without opening a validation database:

```bash
dust db build --offline
dust check --db --offline
```

Offline mode rejects missing entries, changed migrations, changed SQL, changed
fetch shapes, and unsupported cache versions.

> [!NOTE]
> The metadata file lives under `.dart_tool`, so a clean CI runner must restore
> that cache before using `--offline`. Run online validation when no trusted
> cache is available.

## Transactions

Return `Ok` to commit and `Err` to roll back:

```dart
final result = await database.pool.transaction((tx) async {
  return UserDao(tx).createUser('ada@example.com', 'Ada');
});
```

Thrown exceptions also roll back and return a `SqlxDriverError`.

## Dynamic SQL

Use `raw` only when SQL cannot be static, such as an admin-selected table. Raw
SQL is unchecked and uses native SQLite placeholders:

```dart
final result = await database.pool.raw.fetch(
  'SELECT * FROM users WHERE id = ?',
  [id],
);
```

For advanced SQLite-specific operations, access the native database explicitly:

```dart
final sqlite = (database.pool as Sqlite3Executor).database;
final version = sqlite.select('SELECT sqlite_version()').single[0];
```

> [!TIP]
> Prefer generated DAOs for product queries. Keep `raw` and native access small
> because neither path receives Dust's build-time SQL validation.

## Example

The [shopping app database](../../examples/shopping_app/lib/core/db/shopping_cache_database.dart)
demonstrates generated DAOs, migrations, typed rows, JSON columns, converters,
transactions, and cache-backed repository behavior.
