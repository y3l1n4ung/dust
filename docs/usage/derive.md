# Data Classes (Derive)

Dust makes working with immutable data classes in Dart effortless. By annotating your classes with `@Derive`, you can automatically generate essential boilerplate like `toString`, equality checks, and `copyWith` methods.

---

## Installation

Add the core annotation package to your `pubspec.yaml`:

```yaml
dependencies:
  derive_annotation: ^0.1.0
```

---

## Basic Example

Add the `@Derive` annotation with the specific traits you want to generate.

```dart
import 'package:derive_annotation/derive_annotation.dart';

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

> [!IMPORTANT]
> **Requirements for Generation:**
> 1. You **must** include the `part 'filename.g.dart';` directive.
> 2. You **must** add the `with _$ClassName` mixin to your class.
> 3. Your fields should typically be `final` to ensure immutability.

---

## Available Traits

| Trait | What it Generates | Description |
| :--- | :--- | :--- |
| `ToString()` | `toString()` | Returns a string representation including all field names and values. |
| `Eq()` | `==` and `hashCode` | Implements value-based equality. Supports deep equality for collections. |
| `CopyWith()` | `copyWith(...)` | Generates a method to create a new instance with specific fields updated. |

---

## Deep Equality

When using `Eq()`, Dust automatically handles deep equality for standard collection types:
- `List<T>`
- `Set<T>`
- `Map<K, V>`

> [!TIP]
> This removes the need for manual loops or external packages like `collection` when comparing data classes containing nested lists or maps.

---

## Generation Output

Dust generates these members as a Dart mixin. The generated code is injected into your class via the `with` keyword.

```dart
// product.g.dart (Simplified)
mixin _$Product on Product {
  @override
  String toString() => 'Product(id: $id, name: $name, priceCents: $priceCents)';

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Product &&
          id == other.id &&
          name == other.name &&
          priceCents == other.priceCents;

  @override
  int get hashCode => Object.hash(id, name, priceCents);

  Product copyWith({
    String? id,
    String? name,
    int? priceCents,
  }) {
    return Product(
      id: id ?? this.id,
      name: name ?? this.name,
      priceCents: priceCents ?? this.priceCents,
    );
  }
}
```

---

## Migration Guide

**Coming from `freezed` or `equatable`?**

| Feature | `freezed` | `equatable` | Dust |
| :--- | :--- | :--- | :--- |
| Equality | Default | `extends Equatable` | `@Derive([Eq()])` |
| toString | Default | `stringify: true` | `@Derive([ToString()])` |
| CopyWith | Default | N/A | `@Derive([CopyWith()])` |
| Performance | Fast | Medium | **Instant (Rust Engine)** |

> [!NOTE]
> Dust is designed for performance at scale. In large monorepos (500+ models), Dust completes full rebuilds in the time it takes `build_runner` to initialize.
