# Serde Plan

## Goal

Make Dust serde feel close to Rust `serde`: explicit, predictable, strict when
requested, and safe for production JSON boundaries.

## Current State

- `Serialize()` emits `toJson()`.
- `Deserialize()` emits `fromJson(...)`.
- `@SerDe(renameAll: ...)` supports common rename rules.
- Field config supports `rename`, `aliases`, `defaultValue`, `skip`,
  `skipSerializing`, `skipDeserializing`, and unknown-key rejection.
- Nested lists, sets, maps, and known Dust models are supported in the showcase.

## Improvements

- Add stronger diagnostics for unsupported field types.
- Split encode and decode helpers into smaller tested units.
- Preserve key order from source fields.
- Support enum serde with rename rules and unknown value diagnostics.
- Support custom converter annotations for field and type level conversion.
- Support nullable default behavior explicitly.
- Support `DateTime`, `Uri`, and `BigInt` policies.
- Add strict map-key handling for non-string keys.
- Add generated error messages that include JSON key and expected Dart type.

## API Sketch

```dart
@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerdeRename.snakeCase, disallowUnrecognizedKeys: true)
class User with _$UserDust {
  final String id;

  @SerDe(rename: 'display_name', aliases: ['name'])
  final String displayName;

  @SerDe(defaultValue: [])
  final List<String> tags;

  const User({required this.id, required this.displayName, this.tags = const []});
}
```

## Tests

- Golden tests for every rename rule.
- Analyzer tests for generated `toJson` and `fromJson`.
- Runtime tests for aliases, defaults, skips, nested collections, nullable
  fields, unknown keys, malformed input, and custom converters.
- Negative tests for unsupported function and record serialization.

## Done

- Serde output works for common REST API models.
- Errors are deterministic and useful.
- Public Dart annotations have full Dartdoc.
- README shows serde examples and limits.
