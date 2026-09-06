# Changelog

All notable changes to `dust_db_sqlite3` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.1.5]

### Performance

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
