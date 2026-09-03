# Changelog

All notable changes to `dust_db_sqlite3` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.1.4] - 2026-09-03

### Fixed

- Await the transaction control result in `savepoint` and `exclusive`
  transactions so the commit/release completes before returning.

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
