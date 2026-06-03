# derive_serde_annotation

Dust serde annotations for Dart.

This package builds on top of
[`derive_annotation`](../derive_annotation/README.md) and adds the JSON-focused
derive surface used by Dust generation.

It contains:

- `Serialize` for generated `toJson()`
- `Deserialize` for generated `_$TypeFromJson(...)` and enum decode helpers
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
cargo install --git https://github.com/y3l1n4ung/dust dust_cli
```

## Example

```dart
import 'package:dust_dart/serde.dart';

part 'user.g.dart';

@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase)
class User with _$User {
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

## Enum serde

```dart
@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.kebabCase)
enum AccessLevel {
  superAdmin,
  guestUser,
  readOnly,
}
```

Dust generates enum encode/decode helpers. In this example,
`AccessLevel.superAdmin` serializes as `super-admin`, and unknown wire values
throw an `ArgumentError`.

Per-variant serde annotations are not supported yet. Use declaration-level
`renameAll` or a field-level `SerDeCodec` for custom enum wire formats.

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
class AuditLog with _$AuditLog {
  @SerDe(using: unixEpochDateTimeCodec)
  final DateTime createdAt;

  const AuditLog(this.createdAt);

  factory AuditLog.fromJson(Map<String, Object?> json) =>
      _$AuditLogFromJson(json);
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

## Full Usage Guide

See the root usage docs for the end-to-end serde walkthrough:

- [../../docs/usage/serde.md](../../docs/usage/serde.md)

## Generator status

Dust currently uses this package in the Rust generator:

- parser captures declaration, field, and enum annotations
- resolver resolves serde symbols at class, enum, and field level
- `dust_plugin_serde` emits `toJson()`, `_$TypeFromJson(...)`, and enum
  encode/decode helpers

Open work:

- per-variant enum metadata
- type-level codec registration
- custom global scalar policies
