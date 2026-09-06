# Changelog

All notable changes to `dust_db_postgres` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.1.0]

First release. PostgreSQL runtime for generated Database code, wrapping
`package:postgres` the way `dust_db_sqlite3` wraps `package:sqlite3`.

### Added

- `PgPool`, a pool that also runs statements, opened from a connection URL.
- `PostgresExecutor` implementing the five `Executor` primitives, `PgConnectOptions`,
  `PostgresRow`, `PostgresTransaction`, and `PostgresUnsafeSql`.
- Migrations applied in name order inside one transaction, guarded by a
  PostgreSQL advisory lock. Unlike SQLite, where one process holds the file,
  several servers can start against the same database at once.

### Notes

- **The SQL reaches the server unchanged.** Postgres reads `$1` natively, and
  values bind with an unspecified type so the server infers them. Nothing here
  rewrites query text — the SQLite runtime does the opposite, rewriting `$n` to
  `?` at bind time.
- **A Dart `List` binds as a PostgreSQL array**, so `= ANY($1)` needs no
  encoding at the call site. SQLite reaches the same place through `json_each`
  and a driver-side JSON encode.
- **`ExecResult.lastInsertId` is always null.** PostgreSQL has no counterpart to
  SQLite's `last_insert_rowid()`; `RETURNING` is the portable answer.
- **Nested transactions are savepoints.** `package:postgres` exposes no savepoint
  API — a transaction session cannot open a transaction of its own — so this
  package issues `SAVEPOINT`, `RELEASE` and `ROLLBACK TO` itself.
