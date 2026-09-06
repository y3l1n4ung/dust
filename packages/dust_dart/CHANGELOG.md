# Changelog

All notable changes to `dust_dart` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.1.5]

### Added

- `UnsafeSql` — `fetch`, `fetchAs<T>(sql, parameters, mapper)`, and `execute` —
  for the administrative SQL build-time validation cannot reach: migrations,
  `EXPLAIN`, one-off operations.

  It hangs off `DatabaseClient` as `unsafe`, so a generated facade exposes it
  and an executor does not. A request handler is handed an executor, and no cast
  takes an executor to a `DatabaseClient`, which is the difference from `raw`:
  `db as Executor` always succeeded, because every pool, connection and
  transaction implements `Executor`.

  The decoder is passed explicitly rather than resolved from the row type.
  Generated terminals exist only for validated queries, and that asymmetry is
  deliberate — the checked path is the ergonomic one.

### Changed

> [!IMPORTANT]
> **Breaking, and `^0.1.4` upgrades into it.** Every inline query terminal now
> returns `Result<T, SqlxError>` instead of throwing. A pubspec asking for
> `dust_dart: ^0.1.4` resolves to 0.1.5, so an app that pins nothing gets the
> change without asking for it. Run `dust build` to regenerate.

### Changed

- **Database**: the query terminals return `Result`. `QueryAs.fetchOneWith`,
  `fetchOptionalWith` and `fetchAllWith`, `QueryScalar.fetchOne` and
  `fetchOptional`, `QueryRaw.fetch`, `QueryExecute.execute`, and the generated
  `extension $TypeQuery` terminals all hand back `Result<T, SqlxError>`.

  Generated `@SqlxDao` methods have always returned `Result`; the inline path
  wrapped the same executor call in an unwrap that threw
  `StateError('SQL operation failed: ...')`, destroying the typed `SqlxError` on
  the way out. Two paths through the same executor answered a failed query in
  two different ways, and only one of them could be handled.

### Migrating

A call that used the value directly now matches on the result:

```dart
// Before
final user = await queryAs<UserRow>(sql, [id]).fetchOne(db);
print(user.email);

// After
final user = await queryAs<UserRow>(sql, [id]).fetchOne(db);
switch (user) {
  case Ok(:final value):
    print(value.email);
  case Err(:final error):
    print('lookup failed: $error');
}
```

To keep the throwing behavior at one call site, `unwrapOrElse` supplies it:

```dart
final user = (await queryAs<UserRow>(sql, [id]).fetchOne(db))
    .unwrapOrElse((error) => throw StateError('$error'));
```

`@SqlxDao` methods are unaffected — they already returned `Result`.

## [0.1.4] - 2026-09-03

### Added

- `RowDeserializer<T>`, the row-side mirror of `Deserializer<DartT, JsonT>`.
  Reading a row constructs a value, so — exactly as with `Deserialize` — there
  is no instance to declare the method on, and a generated witness object
  carries the capability instead.
- `RowMapperDeserializer<T>`, which wraps a plain `RowMapper<T>` function as a
  `RowDeserializer<T>`, and `RowDeserializerMapper.asMapper`, the reverse view.
  The latter mirrors `DeserializerJsonMirror` on the JSON side.
- `QueryAs.fetchOneWith`, `fetchOptionalWith`, and `fetchAllWith`, which take
  the row mapping as an argument. For a row type Dust does not generate.

### Removed

- **`RowMapperRegistry` and `registerRowMapper` are gone**, along with the
  `registerRowMapper` initializer generated row files used to emit. A
  process-wide `Map<Type, RowMapper>` filled by top-level initializers made a
  missing row mapping a runtime `SqlxError.decode`, and whether it hit depended
  on whether the part file had been imported anywhere in the isolate.
- `QueryAs.fetchOne`, `fetchOptional`, and `fetchAll` are no longer instance
  methods. They are generated per row type instead, as
  `extension $TypeQuery on QueryAs<Type>`. Call sites are unchanged —
  `queryAs<Order>(sql, args).fetchOne(db)` still works — but Dart now resolves
  the terminal from the static type at compile time, so a row type with no
  `@Derive([FromRow()])` has no terminal and the call does not compile. An
  instance member would always beat an extension member, which is why they had
  to move.

### Changed

- Generated row output follows the naming the other derives use. The public
  `extension TypeFromRow on Type` with its `static fromRow` is replaced by a
  private `_$TypeFromRow(Row row)` function, mirroring serde's
  `_$TypeSerialize`, plus the public `$TypeRowDeserializer` witness, mirroring
  `$TypeSerializer`.

#### Migrating

Nothing changes for `@SqlxDao`, and nothing changes for a `queryAs<T>` call
whose `T` derives `FromRow`:

```dart
// unchanged
queryAs<Order>(sql, args).fetchOne(db);
```

Two things move:

```dart
// before — a public generated extension
final order = OrderFromRow.fromRow(row);
// after — the public generated witness
final order = const $OrderRowDeserializer().deserialize(row);

// before — a mapper argument on the query, resolved through the registry
queryAs<Legacy>(sql, args).fetchOne(db);
// after — the mapping goes to the terminal
queryAs<Legacy>(sql, args).fetchOneWith(db, Legacy.fromRow);
```

A row library that uses a `show` clause on `package:dust_dart/db.dart` needs
`QueryAs` and `DatabaseExecutor` added to it.

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
