# derive_annotation

Base Dust derive annotations for Dart.

This package defines the shared annotation contract used by Dust code generation.
It contains:

- the `@Derive([...])` container annotation
- core derive trait markers like `ToString`, `Eq`, and `CopyWith`
- base extension points for future packages such as `derive_serde_annotation`
- a re-export of `package:collection/collection.dart` so generated deep-equality
  support can use collection helpers through the same library import

## Install Dust

Install the Dust CLI before using these annotations.

macOS / Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/y3l1n4ung/dust/main/install.sh | bash
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/y3l1n4ung/dust/main/install.ps1 | iex
```

Or with Cargo:

```bash
cargo install dust_cli
```

## Example

```dart
import 'package:derive_annotation/derive_annotation.dart';

part 'user.g.dart';

@Derive([ToString(), Eq(), CopyWith()])
class User with _$UserDust {
  final String id;
  final String? name;

  const User(this.id, this.name);
}
```

Run Dust:

```bash
dust build
```

Dust writes `user.g.dart` and the `_$UserDust` mixin members for `toString()`,
`==`, `hashCode`, and `copyWith(...)`.

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
- `ToString()` is Dust's public derive marker for generated `toString()`.
