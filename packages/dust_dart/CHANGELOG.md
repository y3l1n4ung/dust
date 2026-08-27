# Changelog

All notable changes to `dust_dart` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.1.4] - 2026-08-27

### Added

- `RowDeserializer<T>`, the row-side mirror of `Deserializer<DartT, JsonT>`.
  Reading a row constructs a value, so — exactly as with `Deserialize` — there
  is no instance to declare the method on and a generated witness object
  carries the capability instead. `@Derive([FromRow()])` now generates
  `$TFromRow`, a `const` witness implementing it.
- `queryAs(..., using:)` and `QueryAs.withDeserializer`, which take that
  witness. Passing `const $OrderFromRow()` makes a row type with no mapping an
  analyzer error, because a type Dust generated nothing for has no `$TFromRow`
  to name. An explicit `mapper:` still wins over `using:`, and both win over
  `RowMapperRegistry`, which stays the fallback and is the only one of the
  three that can fail at runtime.
- `RowDeserializerMapper.asMapper`, the plain-function view, mirroring the
  `DeserializerJsonMirror` extension on the JSON side.

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
