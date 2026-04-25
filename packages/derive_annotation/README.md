# derive_annotation

Base Dust derive annotations for Dart.

This package defines the shared annotation contract used by Dust code generation.
It contains:

- the `@Derive([...])` container annotation
- core derive trait markers like `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`, and `CopyWith`
- base extension points for future packages such as `derive_serde_annotation`
- a re-export of `package:collection/collection.dart` so generated deep-equality
  support can use collection helpers through the same library import

## Example

```dart
import 'package:derive_annotation/derive_annotation.dart';

part 'user.g.dart';

@Derive([Debug(), PartialEq(), Hash(), CopyWith()])
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

## Why `collection` is re-exported

Dust generates `.g.dart` files as `part of` the source library. Part files cannot add
their own imports, so any deep equality helpers used by generated code must already
be visible through the source library's imports. Re-exporting
`package:collection/collection.dart` from `derive_annotation.dart` makes those helpers
available automatically for future deep `Eq` and `Hash` generation.

## Notes

- `PartialEq` and `Eq` are both exposed because Dust is borrowing derive naming
  conventions from other ecosystems.
- The current Dart backend generates one `operator ==` implementation, so
  `PartialEq` and `Eq` are treated the same by code generation today.
