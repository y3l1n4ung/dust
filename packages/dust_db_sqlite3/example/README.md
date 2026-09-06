# Examples

One question per file. Each is under 80 lines, opens its own database, and
prints what it just demonstrated, so the answer is not buried in an application.
Named after the question you would type into a search box — `nullable_columns`,
not `advanced_reading`.

```shell
dart run example/fetch_optional.dart
```

Nothing to set up: SQLite is in the process, so every file here brings its own
database and leaves nothing behind. The two that need a file on disk use a
temporary directory and delete it.

## The contract

Every example in this directory:

1. Opens with a doc comment saying what it answers, and how to run it.
2. Prints one line per point it makes, so [`../test/example`](../test/example)
   can run it and assert on the output rather than only compiling it.
3. Closes the database in a `finally`, because an example is also a template.
4. Answers one question. A second concept belongs in a second file.

The PostgreSQL package has [the same set](../../dust_db_postgres/example) where
the answer differs.

## Connecting

| Question | File |
| :--- | :--- |
| The smallest database that works | [`open_in_memory.dart`](open_in_memory.dart) |
| A database that outlives the process | [`open_a_file.dart`](open_a_file.dart) |
| WAL, pragmas, busy timeout, foreign keys | [`connect_options.dart`](connect_options.dart) |
| A connection that cannot write | [`read_only.dart`](read_only.dart) |
| Changing the schema over time | [`migrations.dart`](migrations.dart) |

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
| Rows affected, and the generated id | [`execute.dart`](execute.dart) |
| Reading the written row back in one statement | [`returning.dart`](returning.dart) |
| Commit on success, revert on failure | [`transactions.dart`](transactions.dart) |
| Undoing part of a transaction | [`savepoints.dart`](savepoints.dart) |

## Values and parameters

| Question | File |
| :--- | :--- |
| Why `$1` and not `?` | [`placeholders.dart`](placeholders.dart) |
| An `IN` list without dynamic SQL | [`set_membership.dart`](set_membership.dart) |
| Storing bytes | [`blobs.dart`](blobs.dart) |
| Timestamps, in a database with no timestamp type | [`dates_and_times.dart`](dates_and_times.dart) |

## When it goes wrong

| Question | File |
| :--- | :--- |
| What a failure looks like when nothing throws | [`error_handling.dart`](error_handling.dart) |
| Telling a duplicate apart from an outage | [`constraint_violations.dart`](constraint_violations.dart) |
| Making SQLite enforce the references you declared | [`foreign_keys.dart`](foreign_keys.dart) |
| SQL that validation never sees, and its price | [`unchecked_sql.dart`](unchecked_sql.dart) |

## All of it together

| Question | File |
| :--- | :--- |
| The shape of a small application | [`dust_db_sqlite3_example.dart`](dust_db_sqlite3_example.dart) |
