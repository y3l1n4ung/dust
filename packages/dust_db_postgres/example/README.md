# dust_db_postgres examples

Each file runs on its own against a PostgreSQL database you name:

```shell
DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
  dart run example/<file>.dart
```

There is no in-memory PostgreSQL, so unlike the SQLite examples these cannot
bring their own database. Each creates the tables it needs and drops them
afterwards.

| Example | What it shows |
| :--- | :--- |
| [`dust_db_postgres_example.dart`](dust_db_postgres_example.dart) | Connecting, migrating, `RETURNING`, and a transaction. |
| [`transactions.dart`](transactions.dart) | Commit on `Ok`, revert on `Err`, and a nested transaction as a savepoint. |
| [`set_membership.dart`](set_membership.dart) | `= ANY($1)` with a bound list, including the empty one. |
