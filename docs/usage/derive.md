# Derive Guide

Use `derive_annotation` when you want Dust to generate object helpers without
JSON-specific behavior.

## Install

Add the package to `pubspec.yaml`:

```yaml
dependencies:
  derive_annotation: ^0.1.0
```

Then fetch packages:

```bash
dart pub get
```

## Minimal Example

Reference file:

- [examples/product_showcase/lib/models/price.dart](../../examples/product_showcase/lib/models/price.dart)

```dart
import 'package:derive_annotation/derive_annotation.dart';

part 'price.g.dart';

@Derive([ToString(), Eq(), CopyWith()])
class Price with _$PriceDust {
  const Price({
    required this.currency,
    required this.cents,
    required this.tags,
  });

  final String currency;
  final int cents;
  final List<String> tags;
}
```

Run generation:

```bash
dust build
```

## What Gets Generated

`ToString()` generates:

- `toString()`

`Eq()` generates:

- `operator ==`
- `hashCode`

`CopyWith()` generates:

- `copyWith(...)`

Dust also handles deep equality for collection fields through the
`derive_annotation` package surface.

## Required Structure

- keep `part 'file_name.g.dart';` in the same file
- add `with _$TypeNameDust` to the class declaration
- run `dust build`, `dust check`, or `dust watch` from the package root

## When To Use Derive Only

Use plain derive when:

- you want immutable model ergonomics
- you do not need JSON conversion
- you want to keep transport concerns out of the model

If you also need JSON conversion, continue with the
[Serde guide](./serde.md).

## See Also

- [Package README](../../packages/derive_annotation/README.md)
- [Usage overview](./README.md)
