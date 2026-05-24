# Product Showcase

This package is the runnable reference project behind the main Dust usage docs.

Read the canonical guides in:

- [../../docs/usage/README.md](../../docs/usage/README.md)
- [../../docs/usage/derive.md](../../docs/usage/derive.md)
- [../../docs/usage/serde.md](../../docs/usage/serde.md)
- [../../docs/usage/http.md](../../docs/usage/http.md)

## What This Package Covers

- derive generation such as `ToString()`, `Eq()`, and `CopyWith()`
- serde generation such as `Serialize()`, `Deserialize()`, enum values, rename rules, defaults, aliases, and codecs
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
- [lib/models/json_codec_bundle.dart](lib/models/json_codec_bundle.dart)
- [lib/api/json_placeholder_api.dart](lib/api/json_placeholder_api.dart)
- [lib/api/todo_api.dart](lib/api/todo_api.dart)

## Related Docs

- [../../README.md](../../README.md)
- [../../docs/developer.md](../../docs/developer.md)
- [../stress_project/README.md](../stress_project/README.md)
