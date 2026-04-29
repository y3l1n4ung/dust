# derive_annotation

Base Dust derive annotations for Dart.

This package defines the shared annotation contract used by Dust code generation.
It contains:

- the `@Derive([...])` container annotation
- core derive trait markers like `ToString`, `Eq`, and `CopyWith`
- base extension points for future packages such as `derive_serde_annotation`
- a re-export of `package:collection/collection.dart` so generated deep-equality
  support can use collection helpers through the same library import

## Example

```dart
import 'package:derive_annotation/derive_annotation.dart';

part 'user.g.dart';

@Derive([ToString(), Eq(), CopyWith()])
class User {
  final String id;
  final String? name;

  const User(this.id, this.name);
}
```

## Extension packages

Future packages can extend the base contract by subclassing `DeriveTrait` or
`DeriveConfig`.

```dart
import 'package:derive_annotation/derive_annotation.dart';

final class Serialize extends DeriveTrait {
  const Serialize();
}

final class SerDe extends DeriveConfig {
  const SerDe();
}
```

## Notes

- `Eq` generates both `operator ==` and matching `hashCode`.
