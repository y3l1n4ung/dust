# Changelog

All notable changes to `dust_db_postgres` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.1.0]

First release. PostgreSQL runtime for generated Database code, wrapping
`package:postgres` the way `dust_db_sqlite3` wraps `package:sqlite3`.

### Added

- `PostgresDriver`, a pool that also runs statements, opened from a connection
  URL, with `PgPool` as the `sqlx-postgres` alias.
- `PostgresExecutor`, `PgConnectOptions`, `PostgresRow`, and
  `PostgresUnsafeSql`.
- `?sslmode=` is read from the connection URL — `disable`, `require` or
  `verify-full` — so a URL that works with `psql` works here. Explicit
  `PgConnectOptions` win over it. libpq's `prefer` and `allow` are rejected
  rather than mapped, since they mean "try TLS, fall back to plaintext" and
  guessing either way would decide something the caller left to the connection.
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
