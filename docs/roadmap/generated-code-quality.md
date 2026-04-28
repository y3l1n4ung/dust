# Generated Code Quality Plan

## Goal

Make Dust output look hand-written: stable, compact, analyzer-clean, and easy to
review in committed `.g.dart` files.

## Current State

- `Debug()` emits `toString()`.
- `Eq()` emits `operator ==` and matching `hashCode`.
- `CopyWith()` emits typed named parameters and deep copies known Dust models and
  nested collections.
- Generated files include `DeepCollectionEquality` only when required.
- Output is deterministic and passes product showcase analyzer/tests.

## Improvements

- Remove redundant local variables when an expression has no transform.
- Prefer direct constructor arguments when no setup is needed.
- Keep local temporary names deterministic and collision-safe.
- Keep generated lines within Dart formatter-friendly boundaries.
- Avoid helper constants/imports unless a generated member actually needs them.
- Use clearer diagnostics when constructor selection fails.
- Emit feature sections in stable order: helpers, mixin self getter, debug,
  equality, copyWith, serde.
- Add a generated-output compatibility test per public derive marker.

## Public API Impact

No new user annotation required. This is output quality and generator
correctness work.

## Tests

- Rust golden tests for each feature and combined feature order.
- Dart analyzer test for every generated output shape.
- Product showcase tests for abstract classes, mixins, extends, nullable fields,
  generic collections, records/functions as passthrough types.
- Cross-platform CI for Linux, macOS, and Windows path behavior.

## Done

- No redundant temp variables in common copyWith output.
- Golden tests cover single-feature and combined-feature generation.
- `dart analyze` passes on all generated showcase files.
- CI blocks regressions.
