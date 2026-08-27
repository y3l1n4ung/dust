# Changelog

All notable changes to `dust_dart` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.1.4] - 2026-08-27

### Added

- `RowDeserializer<T>`, the row-side mirror of `Deserializer<DartT, JsonT>`.
  Reading a row constructs a value, so — exactly as with `Deserialize` — there
  is no instance to declare the method on and a generated witness object
  carries the capability instead. `@Derive([FromRow()])` generates
  `$TypeFromRow`, a `const` witness implementing it.
- `RowMapperDeserializer<T>`, which wraps a plain `RowMapper<T>` function as a
  `RowDeserializer<T>`. The escape hatch for a row type Dust does not generate.
- `RowDeserializerMapper.asMapper`, the plain-function view, mirroring the
  `DeserializerJsonMirror` extension on the JSON side.
- `QueryAs.withDeserializer` and `QueryAs.rowMapper`.

### Removed

- **`RowMapperRegistry` and `registerRowMapper` are gone**, along with the
  `registerRowMapper` initializer that generated row files used to emit. A
  process-wide `Map<Type, RowMapper>` filled by top-level initializers made a
  missing row mapping a runtime `SqlxError.decode`, and whether it hit depended
  on whether the part file had been imported anywhere in the isolate. Nothing
  outside this package's own tests used it: generated DAOs have always passed
  their mapper directly.

### Changed

- **`queryAs<T>` requires `using:`.** With no registry there is no way to get
  from `T` to a decoder — Dart has no static interface members — so the mapping
  is an argument. Pass the generated `const $TypeFromRow()`; a row type Dust
  generated nothing for has no such name, which is what moves the failure from
  the first request to the analyzer. `dust db build` reports it too.
- `QueryAs` takes a single `using:` rather than `mapper:` plus `using:`.
  `withMapper` still takes a function and wraps it.

#### Migrating

```dart
// before — resolved at runtime, threw if nothing had registered
queryAs<Order>(sql, args).fetchOne(db);

// after
queryAs<Order>(sql, args, using: const $OrderFromRow()).fetchOne(db);

// a row type Dust does not generate
queryAs<Order>(sql, args, using: RowMapperDeserializer(myFromRow)).fetchOne(db);
```

Generated `@SqlxDao` methods are unaffected; they never went through the
registry.

## [0.1.3] - 2026-07-28

### Added

- Add source-first JSON serialization and deserialization runtime contracts.
- Add `Validatable` for generated validation APIs.

### Changed

- Keep Dart ecosystem `toJson` and `fromJson` mirrors available through
  extension APIs.
- Refine Database runtime contracts and typed query helper support.

## [0.1.0] - 2026-05-08

### Added

- Initial public release of Dust Dart runtime and annotations.
- Includes functional primitives, derive annotations, validation runtime,
  JSON helpers, HTTP annotations, and SQLx-style DB contracts.
