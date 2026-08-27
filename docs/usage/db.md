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

Create a simple migration at `migrations/0001_create_users.sql`:

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
abstract class AppDatabase implements DatabaseClient {
  factory AppDatabase.open(
    String path, {
    SqliteConnectOptions? options,
  }) = _$AppDatabase.open;

  @override
  DatabaseConnection get connection;
}

@SqlxDao()
abstract final class UserDao {
  const factory UserDao(DatabaseExecutor db) = _$UserDao;

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

Open the database and pass its connection to a DAO:

```dart
final database = AppDatabase.open('app.db');
final users = UserDao(database.connection);

final result = await users.findById(42);
result.match(
  ok: (user) => print(user?.name),
  err: (error) => print('Database error: $error'),
);

await database.connection.close();
```

> [!IMPORTANT]
> Keep row models and database roots in separate libraries. `dust build` owns
> normal derives such as `FromRow`; `dust db build` owns `@SqlxDatabase`,
> `@SqlxDao`, and `@Query` output.

## Migrations

Dust reads migration files in the configured directory in sorted filename order.
DB generation embeds the files that should run during normal startup. At
runtime, SQLite records applied filenames in `__dust_schema_migrations` and
runs only new files.

Simple forward migrations use plain `.sql` names:

```text
migrations/
  0001_create_users.sql
  0002_add_user_avatar.sql
```

SQLx reversible migrations are also supported:

```text
migrations/
  0001_create_users.up.sql
  0001_create_users.down.sql
  0002_add_user_avatar.up.sql
  0002_add_user_avatar.down.sql
```

Example reversible pair:

```sql
-- migrations/0002_add_user_avatar.up.sql
ALTER TABLE users ADD COLUMN avatar_url TEXT;
```

```sql
-- migrations/0002_add_user_avatar.down.sql
CREATE TABLE users_new (
  id INTEGER PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL
);

INSERT INTO users_new (id, email, name)
SELECT id, email, name FROM users;

DROP TABLE users;
ALTER TABLE users_new RENAME TO users;
```

For reversible pairs, Dust validates the pair but applies and embeds only the
`.up.sql` file during normal build and runtime startup. `.down.sql` files are
never applied automatically. Every `.up.sql` file must have a matching
`.down.sql` file, and orphan `.down.sql` files are rejected.

Do not mix `0001_name.sql` with `0001_name.up.sql` or `0001_name.down.sql`.
Use one migration style per migration name.

> [!IMPORTANT]
> Do not edit or rename a migration after it ships. Existing installations have
> already recorded its filename; add a new migration for every schema change.

## SQLite Connection Options

`SqliteConnectOptions` keeps production connection behavior explicit without
exposing `package:sqlite3` open-mode types in application code.

Common path-based production settings:

```dart
final database = AppDatabase.open(
  'app.db',
  options: const SqliteConnectOptions(
    foreignKeys: true,
    busyTimeout: Duration(seconds: 5),
    journalMode: SqliteJournalMode.wal,
    synchronous: SqliteSynchronousMode.normal,
  ),
);
```

In tests, use an in-memory database without creating files:

```dart
final database = AppDatabase.open(
  ':memory:',
  options: const SqliteConnectOptions.memory(foreignKeys: true),
);
```

For an existing read-only file, open without migrations:

```dart
final connection = Sqlite3Driver.connect(
  SqliteConnectOptions.readOnly('snapshot.db'),
);
```

Supported options include create-if-missing, read-only open mode, busy timeout,
foreign keys, journal mode, synchronous mode, and custom pragmas. Custom pragma
names are validated before opening the database.

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

`@Derive([FromRow()])` generates two things: a `TypeFromRow.fromRow(Row row)`
mapper, and `$TypeFromRow`, a `const` witness implementing `RowDeserializer<Type>`.
Both read columns by name through the driver-independent `Row` interface.
Generated DAO methods pass the mapper directly, so normal generated database
code does not depend on side-effect registration or import order.

`RowDeserializer<T>` is the row-side mirror of `Deserializer<DartT, JsonT>`,
and exists for the same reason. `serialize()` can be an instance method because
the value already exists; reading a row *constructs* one, so there is no
instance to declare it on and the capability lives on a separate witness object
instead.

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

For manual typed queries outside generated DAOs, pass the generated witness:

```dart
final user = await queryAs<UserRow>(
  'SELECT id, email, name FROM users WHERE id = ?',
  [id],
  using: const $UserRowFromRow(),
).fetchOne(database.connection);
```

`using:` is worth preferring over `mapper:` because a row type Dust generated
nothing for has no `$TypeFromRow` to name, so the analyzer reports it where you
are typing. `mapper:` takes a plain function and remains the escape hatch for a
row type Dust does not own:

```dart
final user = await queryAs<UserRow>(
  'SELECT id, email, name FROM users WHERE id = ?',
  [id],
  mapper: UserRowFromRow.fromRow,
).fetchOne(database.connection);
```

Passing neither falls back to `RowMapperRegistry`, which resolves at runtime and
throws `SqlxError.decode` on the request that reaches it. `dust db build` rejects
a `queryAs<T>` whose `T` has no row mapping anywhere in the package, so that
throw is a build error rather than a production incident:

```
error: queryAs<Widget> row type has no row mapping. Add `@Derive([FromRow()])`
       to `Widget`, or pass one with `mapper:` or `using:`
```

## Error Context

DB calls return `Result<T, SqlxError>`. The error string stays short for app
developers, and the error also carries structured fields for logs and tests:

- `category`, such as `connection`, `migration`, `query`, `decode`,
  `cardinality`, or `transaction`
- `driver`, when known
- `operation`, such as the SQL string, migration name, or transaction command
- `cause`, when the lower-level driver provided one

Generated DAOs and the SQLite runtime use these categories for common failures:
missing tables, migration errors, decode failures, wrong row counts, closed
connections, and transaction control failures.

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
final result = await database.connection.transaction((tx) async {
  return UserDao(tx).createUser('ada@example.com', 'Ada');
});
```

Thrown exceptions also roll back and return a `SqlxDriverError`. Nested SQLite
transactions use savepoints, so an inner rollback does not undo outer work that
later commits.

```dart
await database.connection.transaction((tx) async {
  await UserDao(tx).createUser('ada@example.com', 'Ada');

  final nested = await tx.transaction<Unit>((nestedTx) async {
    await UserDao(nestedTx).createUser('bad@example.com', 'Bad');
    return Err<Unit, SqlxError>(SqlxError.driver('skip nested work'));
  });

  if (nested.isErr) {
    await UserDao(tx).createUser('grace@example.com', 'Grace');
  }
  return const Ok<Unit, SqlxError>(unit);
});
```

Transaction executors are scope-bound. Do not store `tx` and use it after the
callback returns; operations on a closed transaction return `Err(SqlxError)`.

## Dynamic SQL

Use `raw` only when SQL cannot be static, such as an admin-selected table. Raw
SQL is an advanced escape hatch. It is unchecked and uses native SQLite
placeholders:

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
> Prefer `database.connection` plus generated DAOs for product queries. Keep
> `pool.raw` and native access small because neither path receives Dust's
> build-time SQL validation.

## Example

The [shopping app database](../../examples/shopping_app/lib/core/db/shopping_cache_database.dart)
demonstrates generated DAOs, migrations, typed rows, JSON columns, converters,
transactions, and cache-backed repository behavior.
