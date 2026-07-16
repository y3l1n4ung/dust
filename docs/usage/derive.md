# Data Classes

Dust generates `toString`, value equality, and typed `copyWith` APIs for Dart
classes.

## Add the Package

Install the Dust CLI from the [root guide](../../README.md#installation), then
add the Dart runtime package:

```bash
dart pub add dust_dart
```

## Quick Start

Choose the traits you want with `@Derive`:

```dart
import 'package:dust_dart/derive.dart';

part 'product.g.dart';

@Derive([ToString(), Eq(), CopyWith()])
class Product with _$Product {
  const Product({
    required this.id,
    required this.name,
    required this.priceCents,
  });

  final String id;
  final String name;
  final int priceCents;
}
```

Generate the part file from your package root:

```bash
dust build
```

You can now use the generated members normally:

```dart
const product = Product(id: 'p1', name: 'Desk', priceCents: 12000);

print(product);
final renamed = product.copyWith(name: 'Standing desk');
final sameValue = product ==
    const Product(id: 'p1', name: 'Desk', priceCents: 12000);
```

## Traits

| Trait | Generated API |
| :--- | :--- |
| `ToString()` | A `toString()` implementation containing the class name, field names, and values. |
| `Eq()` | Value-based `operator ==` and a matching `hashCode`. |
| `CopyWith()` | A typed, callable `copyWith` API for replacing selected fields. |

Use only the traits needed by the class.

## Class Requirements

- Add a `part 'filename.g.dart';` directive matching the source filename.
- Add the generated `_$ClassName` mixin to the class.
- A class using `CopyWith()` must be concrete and have a constructor that
  accepts every field.
- Existing superclasses and mixins are supported; place the generated mixin in
  the normal mixin list.
- Dust does not generate derive members for `mixin class` targets.

Fields are usually `final` for data-class usage, but Dust does not require
immutability.

## Value Equality

`Eq()` compares the runtime type and every field. Collections are compared by
contents:

- `List` and `Iterable` compare elements in order.
- `Map` compares keys and values by content.
- `Set` uses unordered deep equality.
- Generated `hashCode` uses the same collection rules.

Nested model fields use their own equality implementation.

## CopyWith Behavior

`CopyWith()` keeps omitted fields and replaces only the arguments you pass:

```dart
final renamed = product.copyWith(name: 'Standing desk');
```

Nullable fields can be cleared explicitly:

```dart
final cleared = note.copyWith(note: null);
```

Copying is shallow. Dust keeps existing object and collection references when
you do not replace them, and it stores replacement values without cloning them.

When a field is another model that also derives `CopyWith()`, Dust generates a
chained helper:

```dart
final moved = profile.copyWith.address(city: 'London');
```

## Generated Files

Dust writes the implementation to the declared `.g.dart` part. Do not edit the
generated file directly.

Use the no-write check in CI:

```bash
dust check
```

Runnable examples:

- [Product showcase models](../../examples/product_showcase/lib/models)
- [Generated derive tests](../../examples/product_showcase/test/generated_derive_models_test.dart)
- [Generated copyWith tests](../../examples/product_showcase/test/generated_copywith_test.dart)
