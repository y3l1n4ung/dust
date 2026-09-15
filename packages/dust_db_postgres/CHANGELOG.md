# Changelog

All notable changes to `dust_db_postgres` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.3.0] - 2026-09-15

### Migrating

- The pool opens up to ten connections where it opened one. Pass
  `PgConnectOptions(maxConnections: 1)` to keep the old size.

### Fixed

- `package:postgres` defaults a pool to one connection, so every statement ran
  in turn and one long transaction stalled every request. The default is now
  `PgConnectOptions.defaultMaxConnections`, ten, as in `sqlx`.

### Added

- `PgConnectOptions.maxConnections`. Zero or less is refused at connect.
- `SqlxError.kind` is filled from SQLSTATE class 23: 23505 unique, 23503
  foreign key, 23502 not null, 23514 check, including a deferred foreign key
  that fails at `COMMIT`.
- The library re-exports `dust_dart`'s `Result` extensions, so code importing
  only this package can still call `unwrapOr` and `andThen` on a query result.

## [0.2.0] - 2026-09-13

First release. PostgreSQL runtime for generated Database code, wrapping
`package:postgres` the way `dust_db_sqlite3` wraps `package:sqlite3`.

### Performance

- Prepared statements are held per pooled connection rather than parsed and
  closed on every call. `Session.execute` sends Parse and waits for it, then
  Bind/Execute, then Close and waits for that — three server round trips where
  a held statement needs one. A single-row `SELECT` against a local server
  measured 1023us that way and 321us reusing a statement, which is the ratio
  the round trips predict. End to end, `fetchOne` went 975us to ~355us and
  `fetchScalar` 1025us to ~305us.

  A statement belongs to the connection that parsed it, so queries run through
  `withConnection` and each connection keeps its own, bounded at 64. A
  transaction does not cache: its statements would be parsed and thrown away
  with it.

  PostgreSQL refuses a held statement whose result type changed under it, which
  a migration applied while the process runs will cause. That one error is
  retried once with a freshly parsed statement, so a schema change costs one
  failed call rather than every call after it.

- A retired connection's held statements are dropped with it. The cache is
  keyed by connection and the pool retires them on its own schedule — an age
  limit, a session limit, an error — so without this the map grew for the life
  of the process, holding statements whose connection was already gone.

- A one-row terminal builds no column-name index unless a name is read.
  There is one row, so there is nothing to share an index with, and
  `fetchScalar` reads column zero and never needs one.

- Row reads resolve column names through one index per result, and typed
  terminals no longer build a list of row adapters before mapping.

### Added

- `PostgresDriver`, a pool that also runs statements, opened from a connection
  URL, with `PgPool` as the `sqlx-postgres` alias.
- `PostgresExecutor`, `PgConnectOptions`, `PostgresRow`, and
  `PostgresUnsafeSql`.
- `PgConnectOptions.maxConnectionAge`, how long the pool keeps a connection
  before retiring it. Worth setting behind a proxy that drops idle connections
  on its own schedule: retiring first means the pool replaces a connection
  rather than handing out one the far end has already closed.
- 23 examples in `example/`, one per question, indexed by `example/README.md`.
- Column names resolve through one index per result rather than
  `ResultRow.toColumnMap()`, which allocates a map of every value for every row.
  Over 20k rows of 12 columns the removed step measured 23-34ms against 9ms.
  The index is lazy, so `fetchScalar` and any `readIndex` build none at all.
- `fetchScalar<T?>` answers `Ok(null)` for a NULL value and for no row, rather
  than a decode or cardinality error. This is what `QueryScalar.fetchOptional`
  asks for, so an aggregate over no rows now reads as optional on both drivers;
  it already behaved this way on SQLite.
- `?sslmode=` is read from the connection URL — `disable`, `require` or
  `verify-full` — so a URL that works with `psql` works here. Explicit
  `PgConnectOptions` win over it. libpq's `prefer` and `allow` are rejected
  rather than mapped, since they mean "try TLS, fall back to plaintext" and
  guessing either way would decide something the caller left to the connection.
- Migrations applied in name order inside one transaction, guarded by a
  PostgreSQL advisory lock. Unlike SQLite, where one process holds the file,
  several servers can start against the same database at once.

### Testing

- 103 tests at 100% line coverage, gated in CI against a `postgres:16` service.
  Everything that needs a server is skipped — and reported as skipped — when
  `DUST_DATABASE_URL` is unset, since there is no in-memory PostgreSQL to fall
  back to.
- Every file in `example/` is run by the suite and asserted on its output. An
  example that compiles but prints the wrong answer is still broken, and only
  running it catches that — it is how the nullable-scalar bug above was found.

### Notes

- Requires `dust_dart` 0.2.0 or newer, and a Dust CLI that generates PostgreSQL
  code. Native targets only: PostgreSQL is reached over a socket, so this does
  not run on the web.

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
