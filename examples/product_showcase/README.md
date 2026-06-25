# Product Showcase

This package is the runnable reference project behind the main Dust usage docs.

Read the canonical guides in:

- [../../docs/usage/README.md](../../docs/usage/README.md)
- [../../docs/usage/derive.md](../../docs/usage/derive.md)
- [../../docs/usage/serde.md](../../docs/usage/serde.md)
- [../../docs/usage/http.md](../../docs/usage/http.md)

## What This Package Covers

- derive generation such as `ToString()`, `Eq()`, and `CopyWith()`
- serde generation such as `Serialize()`, `Deserialize()`, enum values, tagged sealed classes, rename rules, defaults, aliases, and codecs
- validation generation such as `Validate()`, `Length`, `Range`, and generated form-field validator functions
- DB row mapping generation through DB-owned `FromRow()`
- latest Dart-style source syntax that keeps generated code behavior unchanged, including final classes, records, switch expressions, and pattern matching
- generated Dio HTTP clients
- generated request-mapping tests under `test/generated/..._test.dart`
- a real fake-online API example against `https://jsonplaceholder.typicode.com`

## Build

Inside this package:

```bash
dart pub get
dust build
```

From the repository root:

```bash
cargo run -p dust_cli -- build --root examples/product_showcase
```

## Validate

```bash
dart analyze
dart test
```

Optional live smoke coverage:

```bash
DUST_RUN_ONLINE_HTTP_TESTS=1 dart test test/json_placeholder_api_test.dart
```

## Key Reference Files

- [lib/models/price.dart](lib/models/price.dart)
- [lib/models/json_profile.dart](lib/models/json_profile.dart)
- [lib/models/json_enum_bundle.dart](lib/models/json_enum_bundle.dart)
- [lib/models/json_payment_event.dart](lib/models/json_payment_event.dart)
- [lib/models/json_codec_bundle.dart](lib/models/json_codec_bundle.dart)
- [lib/models/latest_dart_derive_showcase.dart](lib/models/latest_dart_derive_showcase.dart)
- [lib/api/json_placeholder_api.dart](lib/api/json_placeholder_api.dart)
- [lib/api/todo_api.dart](lib/api/todo_api.dart)

## Latest Dart Derive Showcase

`lib/models/latest_dart_derive_showcase.dart` proves the current generated code
style works with newer Dart source syntax while using all public derive surfaces:

- `ToString()`
- `Eq()` with generated `hashCode`
- `CopyWith()`
- `Serialize()`
- `Deserialize()`
- `Validate()`
- DB-owned `FromRow()`

The showcase intentionally does not change generated code style. Primary
constructor syntax is tracked separately because the local Puro `3.44.1`
environment currently reports Dart `3.12.1`; primary constructors require later
Dart language support.

## Related Docs

- [../../README.md](../../README.md)
- [../../docs/developer.md](../../docs/developer.md)
- [../benchmark_project/README.md](../benchmark_project/README.md)
