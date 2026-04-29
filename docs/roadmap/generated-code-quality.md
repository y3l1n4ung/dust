# Derive Generated Code Quality Plan

## Goal

Make Dust derive output look hand-written: stable, compact, analyzer-clean,
formatter-friendly, and predictable in committed `.g.dart` files.

The derive system covers:

- `ToString()` -> `toString()`
- `Eq()` -> `operator ==` and matching `hashCode`
- `CopyWith()` -> typed copy helpers with nullable clearing and collection copy

## Current State

- Public Dart API is intentionally small: `ToString`, `Eq`, `CopyWith`.
- `ToString()` is the public marker for generated `toString()`.
- `Eq()` emits both equality and `hashCode`; separate `Hash` and `PartialEq`
  are not exposed.
- `Clone()` was removed from the public API.
- `CopyWith()` supports nullable clearing with `_undefined`.
- `CopyWith()` deep copies known Dust model fields by calling `copyWith()`.
- `CopyWith()` deep copies `List`, `Set`, `Map`, and `Iterable` fields.
- Generated deep equality uses `DeepCollectionEquality` only when needed.
- Product showcase covers abstract classes, extends, mixins, nullable fields,
  nested collections, serde, and analyzer checks.

## Output Standards

Generated derive code must:

- Use deterministic member order.
- Avoid redundant temporary variables when no transform occurs.
- Prefer direct constructor arguments when setup is not needed.
- Keep helper constants scoped and emitted only when used.
- Keep generated expressions readable after `dart format`.
- Avoid `dynamic` unless the user source type is `dynamic`.
- Preserve nullable-clear semantics without analyzer warnings.
- Produce stable output across platforms.
- Keep public generated members documented only when user-facing.

## Desired Output Shape

For simple fields, emit direct constructor values:

```dart
return Product(
  sku: sku ?? _dustSelf.sku,
  title: title ?? _dustSelf.title,
);
```

For transformed fields, use one meaningful local:

```dart
final nextPrice = (price ?? _dustSelf.price).copyWith();
final nextCategories = List<Category>.of(
  (categories ?? _dustSelf.categories).map((item) => item.copyWith()),
);
```

Do not emit source/next pairs when the source exists only to feed one transform
without improving readability.

## ToString Plan

Improve `ToString()` / generated `toString()` output:

- Keep `ToString()` as the canonical public marker for `toString()` generation.
- Split long `toString()` output across multiple string segments when needed.
- Keep field order identical to source order.
- Escape class and field names safely.
- Support empty classes with `ClassName()`.
- Add tests for long field lists and inherited fields.

Example target:

```dart
@override
String toString() {
  return 'Product('
      'sku: ${_dustSelf.sku}, '
      'title: ${_dustSelf.title}, '
      'featured: ${_dustSelf.featured}'
      ')';
}
```

## Eq Plan

Improve `Eq()` output:

- Keep `runtimeType` in both equality and hash.
- Use direct `==` for scalar fields.
- Use ordered deep equality for `List`, `Map`, and `Iterable`.
- Use unordered deep equality for `Set`.
- Emit helper constants only when at least one field needs them.
- Use stable formatting for long equality chains.
- Add negative tests for unsupported equality-sensitive types only if a type
  cannot be compared safely.

Example target:

```dart
@override
bool operator ==(Object other) {
  return identical(this, other) ||
      other is Product &&
          runtimeType == other.runtimeType &&
          other.sku == _dustSelf.sku &&
          _dustDeepCollectionEquality.equals(
            other.categories,
            _dustSelf.categories,
          );
}
```

## Hash Plan

Hash is part of `Eq()`.

Improve hash output:

- Use `Object.hash(...)` for small fixed field counts when it keeps output
  shorter.
- Use `Object.hashAll([...])` for larger classes.
- Use matching deep collection hash helpers for collection fields.
- Keep field order exactly aligned with equality comparisons.
- Add tests proving equal objects always have equal hashes for nested
  collections.

## CopyWith Plan

Improve `CopyWith()` output:

- Remove redundant intermediate variables.
- Keep transformed locals only when they improve readability or avoid repeated
  expressions.
- Preserve nullable clearing using `_undefined`.
- Keep public parameter types analyzer-clean.
- Use stable temp names that cannot collide with user fields.
- Deep copy nested Dust models with `.copyWith()`.
- Deep copy nested collections recursively.
- Preserve `Set` as `Set<T>` and `Iterable` as `List<T>` unless source type
  requires a different constructor policy later.
- Emit diagnostics when a constructor cannot recreate all fields.

Nullable clear target:

```dart
OptionalNote copyWith({
  String? id,
  Object? note = _undefined,
}) {
  return OptionalNote(
    id: id ?? _dustSelf.id,
    note: identical(note, _undefined) ? _dustSelf.note : note as String?,
  );
}
```

Deep copy target:

```dart
Product copyWith({
  String? sku,
  Price? price,
  List<Category>? categories,
}) {
  final nextPrice = (price ?? _dustSelf.price).copyWith();
  final nextCategories = List<Category>.of(
    (categories ?? _dustSelf.categories).map((item) => item.copyWith()),
  );

  return Product(
    sku: sku ?? _dustSelf.sku,
    price: nextPrice,
    categories: nextCategories,
  );
}
```

## Constructor Selection

`CopyWith()` requires a constructor that can rebuild the instance.

Rules:

- Prefer unnamed generative constructor when it covers all fields.
- Support named generative constructors when annotated later.
- Match positional and named params to fields.
- Respect default constructor params.
- Ignore `super.key`.
- Emit diagnostics listing missing fields when no constructor works.

Future annotation:

```dart
@CopyWith(constructor: 'internal')
```

Do not add this option until current constructor inference is fully tested.

## Helper Emission

Emit helpers only when required:

- `_undefined` only for nullable fields in `CopyWith()`.
- `_dustDeepCollectionEquality` only for ordered collections.
- `_dustUnorderedDeepCollectionEquality` only for sets.
- no duplicate helper names across mixed derive and serde output.

Part-file constraint:

- Generated `.g.dart` files cannot add imports.
- Required helper types must be available from user imports or annotation package
  re-exports.
- `derive_annotation` re-exports `collection` for deep equality.

## Formatting Plan

Generated fragments should be formatter-friendly before `dart format`.

Rules:

- Prefer block bodies for long generated members.
- Use expression bodies only for short scalar members.
- Break long constructor calls across lines.
- Break long map/list transforms across lines.
- Keep one generated feature separated by a blank line.
- Run generated showcase through `dart format --set-exit-if-changed` once the
  formatter gate is added.

## Diagnostics Plan

Diagnostics should include:

- class name
- derive marker name
- exact missing constructor fields
- unsupported field type
- source span when parser provides it
- suggested fix when possible

Example:

```text
CopyWith cannot target Product because no constructor accepts fields:
  - categories
  - attributes
Add a constructor parameter for each field or remove CopyWith().
```

## Test Matrix

Rust tests:

- `ToString` for empty, one-field, many-field, inherited-field classes.
- `Eq` for scalar, list, set, map, iterable, nested Dust model fields.
- `CopyWith` for required, optional, nullable-clear, named constructor,
  positional constructor, inherited fields, nested collections, unknown types.
- Combined derives in every order users may write.
- Helper emission with no unused helpers.
- Diagnostic snapshots for invalid constructors and unsupported shapes.

Dart tests:

- Analyzer passes on every generated showcase file.
- Runtime equality/hash contract tests.
- Runtime copyWith clear-vs-retain tests.
- Runtime deep-copy tests proving nested copied objects are not same identity
  where Dust owns the nested model.
- Runtime mixin/extends/abstract-class behavior.

CI gates:

- `cargo fmt --all --check`
- `cargo test --workspace --quiet`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `dart analyze`
- `dart test`
- product showcase generation and analyzer

## Milestones

1. Formatter-safe emit helpers and member layout.
2. CopyWith redundant-local cleanup.
3. ToString block formatting for long output.
4. Eq/hash helper emission and hash strategy cleanup.
5. Constructor diagnostics with missing-field detail.
6. Golden snapshot expansion.
7. Dart formatter gate for generated examples.

## Done

- Generated derive output is readable and stable.
- No redundant source/next temp pairs in common copyWith output.
- Nullable clear semantics stay analyzer-clean.
- Equality and hash behavior match for nested collections.
- Generated files pass Dart analyzer, tests, and formatter gate.
- CI blocks regressions before release.
