# Changelog

All notable changes to `dust_db_sqlite3` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.1.5]

### Added

- `Sqlite3UnsafeSql`, the SQLite implementation of `dust_dart`'s `UnsafeSql`. A
  generated database facade exposes it as `unsafe`.

### Removed

- The `raw` channel on drivers and transactions, with `dust_dart`'s `RawSql`.
  `Sqlite3Executor` implements `DatabaseExecutor` now. Use the facade's
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
