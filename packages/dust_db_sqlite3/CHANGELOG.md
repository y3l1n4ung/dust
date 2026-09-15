# Changelog

All notable changes to `dust_db_sqlite3` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- `SqlxError.kind` is filled from SQLite's extended result codes:
  `SQLITE_CONSTRAINT_UNIQUE` and `SQLITE_CONSTRAINT_PRIMARYKEY` are
  `uniqueViolation`, and the foreign key, not null, and check codes map to
  their kinds, including a deferred foreign key that fails at `COMMIT`.
- The library re-exports `dust_dart`'s `Result` extensions, so code importing
  only this package can still call `unwrapOr` and `andThen` on a query result.

## [0.2.0] - 2026-09-13

### Performance

Compiled ahead of time, which is how a server runs it, against an in-memory
database and best of three:

| Query | Before | After |
| :--- | ---: | ---: |
| `fetchOne`, one row of six columns | 6.17us | 2.82us |
| `fetchScalar` | 3.63us | 1.21us |
| `fetchAll`, 1000 rows of six columns | 673us | 547us |
| A transaction doing one insert | 5.56us | 3.23us |

The changes behind those numbers:

- Prepared statements are held for reuse rather than compiled on every call.
  Preparing was most of what a small query cost: against an in-memory database
  a single-row `SELECT` took 5.7us re-prepared and 2.2us from a held statement,
  so a server answering the same query per request spent more time compiling
  SQL than reading rows. End to end, `fetchOne` went 8.07us to 3.97us and
  `fetchScalar` 4.04us to 1.66us.

  The cache is per connection and shared with transactions over it, keyed by
  SQL text, and bounded at 128 with least-recently-used eviction — Dust's SQL
  is static literals, but unchecked SQL can be assembled at the call site and an
  unbounded cache would grow with the request count.

- A one-row terminal builds no column-name index unless a name is read.
  There is one row, so there is nothing to share an index with, and
  `fetchScalar` reads column zero and never needs one: 1.52us to 1.44us.

- `BEGIN`, `COMMIT` and `ROLLBACK` are held like any other statement. Those
  three strings never vary, so a transaction per request was compiling two of
  them every time — about 0.7us of a 3.6us transaction, now 3.1us. Savepoints
  stay uncached: their names carry a counter, so each is a statement seen once
  and holding them would fill the cache with entries that can never hit.

- Rows are read out of the result set's own data rather than through the
  driver's `Row`, whose constructor copies that data with `List.unmodifiable`
  for every row. Column names resolve through one index built for the whole
  result. Over 1000 rows of six columns the copy and the lookups through it
  measured about 90us, over half of what the adapter cost above the driver.

- Reading a column tests the value against the type asked for before anything
  else, which is what almost every read is. The `num` special case it made
  redundant is gone, so a read is one type test rather than three. Mapping six
  columns over 1000 rows spent 249us above the driver and now spends 202us.

- A statement whose placeholders read `$1, $2, ...` in order binds the caller's
  argument list as it stands rather than copying it into bind order. That is
  nearly every statement, and whether it holds was settled once when the
  rewrite was cached rather than on each call.

- Typed terminals no longer build a list of row adapters before mapping.
  `fetchOne` and `fetchScalar` wrap the one row they read, and `fetchAll` maps
  straight out of the result set. Only unchecked SQL still materialises rows,
  because rows are what it returns.

### Added

- 23 examples in `example/`, one per question, indexed by `example/README.md`:
  connecting, reading, writing, values, and what a failure looks like. The
  package shipped one 67-line file before this.

### Testing

- Coverage 92% to 97%, with the floor raised to match. The new tests are the
  ones that were missing rather than filler: every `SqliteConnectOptions`
  validation rule, each journal and synchronous mode, a database used after
  closing, a migration SQLite refuses, and the control-statement failures a
  transaction reports rather than throws.
- Every file in `example/` is run by the suite and asserted on its output. An
  example that compiles but prints the wrong answer is still broken, and only
  running it catches that.

### Added

- The driver rewrites `$n` placeholders to SQLite's `?` at bind time, reordering
  and repeating arguments as the statement reads them. Generated code used to do
  this, and only for `@SqlxDao` methods, so the same `$1` meant one thing in a
  DAO and another in an inline query. A repeated `$1` now binds its argument
  twice on either path.

  The scanner is ported from the one `dust_db_plugin` checks with at build time,
  case for case: string literals, quoted identifiers with doubled-quote escapes,
  both comment forms, and Postgres dollar-quoted bodies are copied through
  untouched. Rewrites are cached per statement text.

- `Sqlite3UnsafeSql`, the SQLite implementation of `dust_dart`'s `UnsafeSql`. A
  generated database facade exposes it as `unsafe`.

### Removed

- The `raw` channel on drivers and transactions, with `dust_dart`'s `RawSql`.
  `Sqlite3Executor` implements `Executor` now. Use the facade's
  `unsafe`, or `Sqlite3UnsafeSql(driver)` where no facade exists.

- A `List` argument is bound as JSON text, so a set membership test is one
  placeholder over constant SQL:

  ```dart
  queryAs<Order>(
    'SELECT id, item FROM orders WHERE id IN (SELECT value FROM json_each(?))',
    [ids],
  ).fetchAll(pool);
  ```

  SQLite has no array type and `package:sqlite3` will not bind a nested `List`,
  so callers previously reached for dynamic `IN (?, ?, ?)` — the most common
  reason to build SQL by hand. The SQL above is constant and describable, so
  build-time validation covers it.

  A `Uint8List` is left alone: it is how a BLOB is bound, and encoding it would
  write a JSON array of byte values in place of the bytes. A list holding a
  value JSON cannot represent returns `Err(SqlxError.query)` naming the cause.

## [0.1.4] - 2026-09-03

Released alongside Dust 0.1.4. No behavior changes.

### Changed

- Runtime constraint raised to `dust_dart: ^0.1.4`. 0.1.4 removes
  `RowMapperRegistry`, `registerRowMapper`, and the `QueryAs` instance
  terminals, and `^0.1.3` resolved to either side of that.
- `runRoot` and `runSavepoint` return `await result.match(...)` rather than the
  future itself, so the transaction's `try` covers the commit and release path
  it wraps. Both control statements already ran before the scope was
  deactivated — the callbacks are invoked synchronously by `match` — so this
  fixes no observed defect and changes no behavior.

## [0.1.3] - 2026-07-28

### Added

- Add SQLite connection options for foreign keys and busy timeouts.

### Changed

- Harden transaction, migration, row, and error handling behavior.
- Align the driver package with the v0.1.3 `dust_dart` DB contracts.

## [0.1.0] - 2026-05-08

### Added

- Initial public release of the SQLite runtime for generated Database code
  generation.
- Includes pool, transaction, migration, row, and raw SQL executor support.
- Depends on `dust_dart` `0.1.x` DB runtime contracts.
