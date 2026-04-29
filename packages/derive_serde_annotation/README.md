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

## Example

```dart
import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'user.g.dart';

@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase)
class User {
  @SerDe(rename: 'user_id')
  final String userId;

  @SerDe(defaultValue: <String>[])
  final List<String> tags;

  const User(this.userId, this.tags);
}
```

## Design notes

- The package re-exports `derive_annotation`, so one import is enough for
  `Derive`, core derive traits, and serde traits.
- `SerDe` can be used on declarations and fields, like Rust's `#[serde(...)]`.
- `Serialize` and `Deserialize` stay separate derive traits, which maps cleanly
  to Dust's existing trait/plugin architecture.
- The package only defines annotation metadata. Code generation still lives in
  Dust Rust crates.

## Planned Rust integration

The next Dust layers will use this package in three steps:

1. parser captures field annotations
2. resolver resolves serde symbols at class and field level
3. `dust_plugin_serde` emits `toJson()` and `fromJson(...)`
