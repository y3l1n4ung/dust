# Examples

One question per file. Each is under 90 lines, creates the tables it needs and
drops them afterwards, and prints what it just demonstrated, so the answer is
not buried in an application. Named after the question you would type into a
search box — `nullable_columns`, not `advanced_reading`.

```shell
DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
  dart run example/fetch_optional.dart
```

There is no in-memory PostgreSQL, so unlike the SQLite examples these cannot
bring their own database. Each names the one it needs and says so when
`DUST_DATABASE_URL` is unset, rather than failing with a connection error.

A local server is enough:

```shell
docker run --rm -d --name dust-pg -p 5432:5432 \
  -e POSTGRES_PASSWORD=dust -e POSTGRES_USER=dust -e POSTGRES_DB=dust postgres:16
```

## The contract

Every example in this directory:

1. Opens with a doc comment saying what it answers, and how to run it.
2. Prints one line per point it makes, so [`../test/example`](../test/example)
   can run it and assert on the output rather than only compiling it.
3. Owns tables prefixed `example_`, dropped in a `finally`, so a failed run
   leaves nothing for the next one to trip over.
4. Answers one question. A second concept belongs in a second file.

The SQLite package has [the same set](../../dust_db_sqlite3/example) where the
answer differs, and files of its own for what SQLite has to work around.

## Connecting

| Question | File |
| :--- | :--- |
| Opening a pool, and what `sslmode` decides | [`connect.dart`](connect.dart) |
| Timeouts, TLS, and naming yourself in `pg_stat_activity` | [`connect_options.dart`](connect_options.dart) |
| Changing the schema, from more than one instance | [`migrations.dart`](migrations.dart) |

## Reading

| Question | File |
| :--- | :--- |
| A row that must exist | [`fetch_one.dart`](fetch_one.dart) |
| A row that may not | [`fetch_optional.dart`](fetch_optional.dart) |
| Every row a query selects | [`fetch_all.dart`](fetch_all.dart) |
| One value out of one column | [`fetch_scalar.dart`](fetch_scalar.dart) |
| Turning a row into a Dart object | [`row_mappers.dart`](row_mappers.dart) |
| Columns that can be NULL | [`nullable_columns.dart`](nullable_columns.dart) |

## Writing

| Question | File |
| :--- | :--- |
| Rows affected, and why there is no `lastInsertId` | [`execute.dart`](execute.dart) |
| Reading the written row back in one statement | [`returning.dart`](returning.dart) |
| Commit on success, revert on failure | [`transactions.dart`](transactions.dart) |
| Undoing part of a transaction | [`savepoints.dart`](savepoints.dart) |

## Values and parameters

| Question | File |
| :--- | :--- |
| Why `$1`, and why a raw string | [`placeholders.dart`](placeholders.dart) |
| An `IN` list without dynamic SQL | [`set_membership.dart`](set_membership.dart) |
| A column that holds a list | [`arrays.dart`](arrays.dart) |
| A column that holds a document | [`json_columns.dart`](json_columns.dart) |
| Points in time, and why `timestamptz` | [`timestamps.dart`](timestamps.dart) |
| Storing bytes | [`bytea.dart`](bytea.dart) |

## When it goes wrong

| Question | File |
| :--- | :--- |
| What a failure looks like when nothing throws | [`error_handling.dart`](error_handling.dart) |
| Telling a duplicate apart from an outage | [`constraint_violations.dart`](constraint_violations.dart) |
| SQL that validation never sees, and its price | [`unchecked_sql.dart`](unchecked_sql.dart) |

## All of it together

| Question | File |
| :--- | :--- |
| The shape of a small application | [`dust_db_postgres_example.dart`](dust_db_postgres_example.dart) |
