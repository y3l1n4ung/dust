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

## Open A Database

```dart
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

final pool = Sqlite3Pool.open('app.db');
```

Pass the pool to generated DAO/database code that expects
`package:dust_dart/db.dart` `Executor`.

## Migrations

Migrations run once in key order during open:

```dart
final pool = Sqlite3Pool.open(
  'app.db',
  migrations: const {
    '0001_create_users.sql': '''
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL
);
''',
  },
);
```

Applied migration names are stored in `__dust_schema_migrations`.

## Transactions

Use the shared Database transaction contract:

```dart
final result = await pool.transaction((tx) {
  return tx.execute(
    'INSERT INTO users (name) VALUES (?)',
    ['Ada'],
  );
});
```

Transactions commit on `Ok` and roll back on `Err` or thrown exceptions.

## Raw SQL

Generated DAOs use typed executor methods. Handwritten escape hatches can use
`raw`:

```dart
final rows = await pool.raw.fetch(
  'SELECT id, name FROM users WHERE id = ?',
  [1],
);
```

## Exports

- `Sqlite3Driver`
- `SqlitePool`
- `Sqlite3Executor`
- `Sqlite3Row`
