# dust_db_postgres

PostgreSQL runtime for [Dust](https://github.com/y3l1n4ung/dust) Database.

Dust validates your SQL at build time and generates the row mapping. This
package executes it, wrapping `package:postgres` — it is an adapter, not a
database driver of its own.

```dart
import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';

part 'app_database.g.dart';

@SqlxDatabase(type: SqlxDatabaseType.postgres, migrations: './migrations')
abstract class AppDatabase implements DatabaseClient {
  factory AppDatabase.connect(String url, {PgConnectOptions? options}) =
      _$AppDatabase.connect;

  @override
  Connection get connection;
}
```

## What differs from SQLite

The query text does not. `$1` is what you write on either dialect: PostgreSQL
reads it natively, and `dust_db_sqlite3` rewrites it to `?` at bind time.

| | PostgreSQL | SQLite |
| :--- | :--- | :--- |
| Set membership | `= ANY($1)` with a `List` | `IN (SELECT value FROM json_each($1))` |
| New row's id | `RETURNING` | `RETURNING`, or `ExecResult.lastInsertId` |
| Nested transaction | savepoint, issued by this package | savepoint, issued by the driver |
| `boolean` | a real type | `0` and `1`, interpreted |
| Timestamps | `timestamptz`, decoded | ISO-8601 text, parsed |

## Testing

`dart test` runs everything that needs no server. The integration suite is
skipped unless `DUST_DATABASE_URL` points at a database it may write to:

```bash
DUST_DATABASE_URL='postgres://user:pass@localhost:5432/dust_test?sslmode=disable' dart test
```

`?sslmode=` is read from the URL, as every other PostgreSQL tool reads it —
`disable`, `require` or `verify-full`. Explicit `PgConnectOptions` win over it.
libpq's `prefer` and `allow` are rejected rather than guessed: they mean "try
TLS, fall back to plaintext", and this driver has no such mode.

There is no in-memory PostgreSQL, which is also why `dust db build` needs a
server and CI validates from the committed query cache instead.
