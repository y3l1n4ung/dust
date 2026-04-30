# derive_serde_annotation

Dust serde annotations for Dart.

This package builds on top of
[`derive_annotation`](../derive_annotation/README.md) and adds the JSON-focused
derive surface used by Dust generation.

It contains:

- `Serialize` for generated `toJson()`
- `Deserialize` for generated `fromJson(...)`
- `SerDe(...)` for Rust-serde-style declaration and field metadata
- `SerDeRename` for automatic rename rules
- `SerDeCodec<DartT, JsonT>` for custom field codecs

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
import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'user.g.dart';

@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase)
class User with _$UserDust {
  @SerDe(rename: 'user_id')
  final String userId;

  @SerDe(defaultValue: <String>[])
  final List<String> tags;

  const User(this.userId, this.tags);

  factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
}
```

Run Dust:

```bash
dust build
```

Dust writes `user.g.dart`, a generated `toJson()` mixin member, and the
`_$UserFromJson(...)` helper used by the forwarding factory.

## Custom codec

```dart
final class UnixEpochDateTimeCodec implements SerDeCodec<DateTime, int> {
  const UnixEpochDateTimeCodec();

  @override
  int serialize(DateTime value) => value.millisecondsSinceEpoch;

  @override
  DateTime deserialize(int value) =>
      DateTime.fromMillisecondsSinceEpoch(value, isUtc: true);
}

const unixEpochDateTimeCodec = UnixEpochDateTimeCodec();

@Derive([Serialize(), Deserialize()])
class AuditLog {
  @SerDe(using: unixEpochDateTimeCodec)
  final DateTime createdAt;

  const AuditLog(this.createdAt);
}
```

## Design notes

- The package re-exports `derive_annotation`, so one import is enough for
  `Derive`, core derive traits, and serde traits.
- `SerDe` can be used on declarations and fields, like Rust's `#[serde(...)]`.
- `SerDeCodec` is the extension point for custom field conversion.
- `Serialize` and `Deserialize` stay separate derive traits, which maps cleanly
  to Dust's existing trait/plugin architecture.
- The package only defines annotation metadata. Code generation still lives in
  Dust Rust crates.

## Planned Rust integration

The next Dust layers will use this package in three steps:

1. parser captures field annotations
2. resolver resolves serde symbols at class and field level
3. `dust_plugin_serde` emits `toJson()` and `fromJson(...)`
