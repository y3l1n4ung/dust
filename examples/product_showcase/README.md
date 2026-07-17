# Product Showcase

Runnable Dart package that demonstrates Dust's core generated APIs.

Use this example when you want to see annotations, generated files, and tests in
one small package without Flutter app code around it.

## What It Covers

| Area | Example files |
| :--- | :--- |
| Data classes | [`lib/models/product.dart`](lib/models/product.dart), [`lib/models/price.dart`](lib/models/price.dart) |
| JSON | [`lib/models/json_profile.dart`](lib/models/json_profile.dart), [`lib/models/json_codec_bundle.dart`](lib/models/json_codec_bundle.dart) |
| Sealed JSON | [`lib/models/json_payment_event.dart`](lib/models/json_payment_event.dart), [`lib/models/json_untagged_event.dart`](lib/models/json_untagged_event.dart) |
| Validation | [`lib/models/latest_dart_derive_showcase.dart`](lib/models/latest_dart_derive_showcase.dart) |
| DB row mapping | [`lib/models/latest_dart_derive_showcase.dart`](lib/models/latest_dart_derive_showcase.dart) |
| HTTP clients | [`lib/api/todo_api.dart`](lib/api/todo_api.dart), [`lib/api/json_placeholder_api.dart`](lib/api/json_placeholder_api.dart) |

The generated files are committed so the package can be inspected without
running Dust first.

## Run It

From this package:

```bash
dart pub get
dust build
dust check
dart analyze
dart test
```

From the repository root:

```bash
cargo run -q -p dust_cli -- build --root examples/product_showcase
cargo run -q -p dust_cli -- check --root examples/product_showcase
```

The normal test suite is offline. To run the live JSONPlaceholder smoke test:

```bash
DUST_RUN_ONLINE_HTTP_TESTS=1 dart test test/json_placeholder_api_test.dart
```

## Generated HTTP Tests

The HTTP examples enable `generateTest: true`, so Dust also writes request
mapping tests under [`test/generated`](test/generated). These tests verify URL,
method, path, query, header, body, form, multipart, and streaming behavior
without calling the network.

## Latest Dart Syntax

[`lib/models/latest_dart_derive_showcase.dart`](lib/models/latest_dart_derive_showcase.dart)
keeps one model that combines:

- `ToString()`
- `Eq()`
- `CopyWith()`
- `Serialize()`
- `Deserialize()`
- `Validate()`
- `FromRow()`

It also uses final classes, records, switch expressions, and pattern matching in
handwritten Dart code. That proves the parser accepts modern Dart source around
Dust annotations while generated behavior stays normal.

## More Docs

- [Root README](../../README.md)
- [Usage guide](../../docs/usage/README.md)
- [Data class guide](../../docs/usage/derive.md)
- [JSON guide](../../docs/usage/serde.md)
- [HTTP guide](../../docs/usage/http.md)
- [Database guide](../../docs/usage/db.md)
- [Benchmark project](../benchmark_project/README.md)
