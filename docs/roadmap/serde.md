# SerDe Roadmap

## Goal

Make Dust SerDe feel close to Rust `serde`: explicit, predictable, strict when
requested, and safe for production JSON boundaries.

## Scope

This track covers:

- `Serialize()` and generated `toJson()`
- `Deserialize()` and generated `_$TypeFromJson(...)` helpers
- `@SerDe(...)` declaration-level and field-level metadata
- key rename rules, aliases, defaults, skip rules, and unknown-key rejection
- built-in scalar policies for `DateTime`, `Uri`, and `BigInt`
- custom field conversion through `SerDeCodec<DartT, JsonT>`

This track does not yet cover:

- enum serialization
- union/tagged enum strategies
- type-level codecs
- non-JSON wire formats

## Current State

Dust SerDe currently supports:

- [x] `Serialize()` -> generated `toJson()` mixin member
- [x] `Deserialize()` -> generated `_$TypeFromJson(Map<String, Object?> json)`
- [x] `@SerDe(renameAll: ...)` for common Rust-style naming strategies
- [x] field-level `rename`, `aliases`, `defaultValue`, `skip`,
  `skipSerializing`, `skipDeserializing`, and `using`
- [x] declaration-level `disallowUnrecognizedKeys`
- [x] built-in scalar conversion for `DateTime`, `Uri`, and `BigInt`
- [x] nested `List`, `Set`, `Map<String, T>`, and known Dust models
- [x] custom field conversion through `SerDeCodec`
- [x] generated decode diagnostics that include the failing JSON key and
  expected Dart type for built-in conversions
- [x] compact decode output for simple fields, with temporaries kept only when
  aliases or fallback resolution require them
- [x] stronger lowering diagnostics for malformed or suspicious `using:` values

## Public API

### Basic model

```dart
@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)
class User with _$UserDust {
  final String id;

  @SerDe(rename: 'display_name', aliases: ['name'])
  final String displayName;

  @SerDe(defaultValue: [])
  final List<String> tags;

  const User({
    required this.id,
    required this.displayName,
    this.tags = const [],
  });
}
```

### Custom codec

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
class AuditLog with _$AuditLogDust {
  @SerDe(using: unixEpochDateTimeCodec)
  final DateTime createdAt;

  const AuditLog(this.createdAt);
}
```

## Generated Output Rules

Generated SerDe code should:

- keep key order stable from source field order
- stay analyzer-clean without nullable warnings
- use readable multiline expressions for long ternaries and nested transforms
- emit only one obvious encode/decode path per field
- keep helper APIs small and predictable
- preserve user-facing field types in generated signatures

Generated deserialization currently adds a one-line copy hint above helpers:

```dart
// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
```

This is only a hint. Dust still cannot inject a real `User.fromJson(...)`
factory into the source class with the current `part`-file generator model.

## Validation Rules

Dust should reject or flag:

- unresolved field types
- function and record fields for built-in SerDe generation
- `skipDeserializing` without `defaultValue`
- unsupported generic named types without a custom codec
- unsupported class-level `SerDe(...)` options used in field-only positions
- unsupported field-level `SerDe(...)` options used in class-only positions

For codec-backed fields:

- Dust trusts the `SerDeCodec` contract for conversion logic
- Dust still owns nullable wrapping outside the codec
- the codec object must be valid Dart source in the annotation argument

## Open Backlog

### Generator Work

- [ ] support enum SerDe with rename rules and unknown-value diagnostics
- [ ] support type-level codec registration beyond field-level `using`
- [ ] support configurable `DateTime`, `Uri`, and `BigInt` policies beyond the
  current defaults
- [ ] make generated SerDe output stable under `dart format` across showcase
  models
- [ ] decide whether public guidance should require `const` codec objects

### Test Work

- [ ] add golden coverage for every rename rule
- [ ] add negative coverage for unsupported record serialization
- [x] add negative coverage for malformed `using:` values

## Tests

This track needs:

- [x] Rust plugin tests for supported rename/default/skip/codec output
- [x] Rust driver tests for real generated `.g.dart` output
- [x] analyzer tests for generated `toJson()` and `_$TypeFromJson(...)`
- [x] runtime tests for aliases, defaults, skip rules, nested collections,
  nullable fields, unknown-key rejection, built-in scalar conversion, and
  custom `SerDeCodec` fields
- [x] dedicated malformed-input tests with key-aware decode diagnostics
- [x] negative tests for unsupported function serialization
- [ ] negative tests for unsupported record serialization

## Release Criteria

SerDe is in a good release state when:

- [ ] common REST-style models generate cleanly without manual patching
- [ ] generated code is stable after `dart format`
- [ ] diagnostics are deterministic and actionable
- [x] public annotation APIs have Dartdoc
- [x] the product showcase covers real nested and codec-backed models
