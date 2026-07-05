# dust_db_sqlite3

SQLite runtime for generated Database code.

You focus on product. We focus on performance.

## Status

- `0.1.0` is the first public SQLite driver release for Database.
- The package provides a SQLite-backed `Executor` for generated DAO code from
  `package:dust_dart/db.dart`.
- Database is raw SQL first. It is not an ORM and does not provide a query
  builder.
- Publish order matters: publish `dust_dart` before this package.

## Install

Add the SQLite driver next to `dust_dart`:

```yaml
dependencies:
  dust_dart: ^0.1.0
  dust_db_sqlite3: ^0.1.0
```

## Migration Workflow

Create migration files in the app package, usually with
[SQLx CLI](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md):

```sh
cargo install sqlx-cli
sqlx migrate add create_users
```

Dust `0.1.x` expects SQLx's default simple migration files
(`<timestamp>_<name>.sql`) rather than reversible `*.up.sql` / `*.down.sql`
pairs.

Put schema SQL in the generated migration:

```sql
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL
);
```

Dust validates DAO queries against the migration directory during
`dust db build`. Generated database openers embed those files and
`SqlitePool.open` applies unapplied migrations once, recording names in
`__dust_schema_migrations`.

## How to annotate

Use `package:dust_dart/db.dart` annotations for generated Database code and
`dust_db_sqlite3` for the runtime driver:

```dart
import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

import 'user_row.dart';

part 'app_database.g.dart';

@SqlxDatabase(type: SqlxDatabaseType.sqlite, migrations: './migrations')
abstract class AppDatabase {
  factory AppDatabase.open(String path) = _$AppDatabase.open;

  Pool get pool;
}

@SqlxDao()
abstract final class UserDao {
  const factory UserDao(Executor db) = _$UserDao;

  @Query(r'SELECT id, email, name FROM users WHERE id = $1')
  Future<Result<UserRow?, SqlxError>> findById(int id);

  @Query(r'INSERT INTO users (email, name) VALUES ($1, $2)')
  Future<Result<ExecResult, SqlxError>> create(String email, String name);
}
```

Keep row DTOs in normal Dust generation:

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

Run both generation steps from your app package root:

```sh
dust build
dust db build
```

Open the generated database and pass its pool to DAOs:

```dart
final db = AppDatabase.open('app.db');
final pool = db.pool;
final users = UserDao(pool);
```

Full guides:

- [Database annotations and DAO queries](https://github.com/y3l1n4ung/dust/blob/main/docs/usage/db.md)
- [Data classes and row mapping](https://github.com/y3l1n4ung/dust/blob/main/docs/usage/derive.md)
- [Package example](https://github.com/y3l1n4ung/dust/blob/main/packages/dust_db_sqlite3/example/dust_db_sqlite3_example.dart)

## Transactions

Use the shared Database transaction contract:

```dart
final result = await pool.transaction((tx) {
  return tx.execute(
    'INSERT INTO users (email, name) VALUES (?, ?)',
    ['ada@example.com', 'Ada'],
  );
});
```

Transactions commit on `Ok` and roll back on `Err` or thrown exceptions.

## Raw SQL

Generated DAOs use typed executor methods. Handwritten escape hatches can use
`raw`:

```dart
final rows = await pool.raw.fetch(
  'SELECT id, email, name FROM users WHERE id = ?',
  [1],
);
```

## Exports

- `Sqlite3Driver`
- `SqlitePool`
- `Sqlite3Executor`
- `Sqlite3Row`

## Platform note

This package uses native SQLite through `package:sqlite3` and `dart:ffi`.
It is intended for native Dart and Flutter targets, not WASM-first web
deployments.
