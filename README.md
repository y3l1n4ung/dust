# Dust

![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)
![Dart](https://img.shields.io/badge/Dart-3-blue?logo=dart)
![License](https://img.shields.io/badge/License-MIT-green)

Rust-based Dart code generator, inspired by Rust macros.

## Example

```dart
import 'package:derive_annotation/derive_annotation.dart';
import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'user.g.dart';

@Derive([Debug(), Clone(), CopyWith(), Eq(), Hash()])
@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerdeRename.snakeCase)
class User with _$UserDust {
  final String id;
  final String displayName;
  final List<String> tags;

  const User(this.id, this.displayName, this.tags);
}
```

Generated output includes:

- `toString`
- `clone`
- `copyWith`
- `==` and `hashCode`
- `toJson`
- `fromJson`

## Install

Install with Cargo:

```bash
cargo install --path crates/dust_cli
```

Install from this repository with the local installer:

```bash
./install.sh
```

PowerShell:

```powershell
./install.ps1
```

Dart annotation packages:

```yaml
dependencies:
  derive_annotation:
    path: packages/derive_annotation
  derive_serde_annotation:
    path: packages/derive_serde_annotation
```

## Usage

```bash
dust build
```

```bash
dust watch
```

## License

MIT. See [LICENSE](LICENSE). Copyright (c) 2026 Ye Lin Aung.
