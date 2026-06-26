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
- enum serialization and deserialization
- tagged and untagged sealed class serialization and deserialization
- custom field conversion through `SerDeCodec<DartT, JsonT>`

This track does not yet cover:

- variant-level enum metadata such as per-variant `rename` or `skip`
- generated sealed variants with additional derives such as `CopyWith()` or
  `Eq()`
- nullable named sealed-factory parameters that must still be marked
  `required`
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
- [x] enum `Serialize()` / `Deserialize()` helpers
- [x] enum `@SerDe(renameAll: ...)` wire names
- [x] enum values inside nullable fields, lists, sets, and maps
- [x] unknown enum wire value diagnostics
- [x] tagged and adjacent-tagged sealed class helpers
- [x] untagged sealed class helpers
- [x] generated concrete sealed variant classes from redirecting factories
- [x] custom field conversion through `SerDeCodec`
- [x] generated decode diagnostics that include the failing JSON path and
  expected Dart type for built-in conversions
- [x] compact decode output for simple fields, with temporaries kept only when
  aliases or fallback resolution require them
- [x] stronger lowering diagnostics for malformed or suspicious `using:` values

## Public API

### Basic model

```dart
@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)
class User with _$User {
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
class AuditLog with _$AuditLog {
  @SerDe(using: unixEpochDateTimeCodec)
  final DateTime createdAt;

  const AuditLog(this.createdAt);
}
```

### Enum serde

```dart
@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.kebabCase)
enum AccessLevel {
  superAdmin,
  guestUser,
  readOnly,
}

@Derive([Serialize(), Deserialize()])
enum ReviewState {
  pending,
  approved,
  archived,
}

@Derive([Serialize(), Deserialize()])
class RoleBundle with _$RoleBundle {
  const RoleBundle({
    required this.primaryLevel,
    required this.states,
  });

  factory RoleBundle.fromJson(Map<String, Object?> json) =>
      _$RoleBundleFromJson(json);

  final AccessLevel primaryLevel;
  final Set<ReviewState> states;
}
```

`AccessLevel.superAdmin` serializes as `super-admin`. Unknown enum wire values
throw an `ArgumentError` that names the enum type. Per-variant annotations are
not implemented yet; enum wire names currently come from declaration-level
`renameAll` or the variant identifier.

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

- [x] support enum SerDe with rename rules and unknown-value diagnostics
- [ ] support variant-level enum metadata (`rename`, `skip`, aliases/defaults
  if a clear Dart API is chosen)
- [ ] support type-level codec registration beyond field-level `using`
- [ ] support configurable `DateTime`, `Uri`, and `BigInt` policies beyond the
  current defaults
- [x] generate omitted concrete sealed variant classes from redirecting
  factories
- [ ] preserve explicit `required` on nullable sealed-factory parameters
- [x] make generated SerDe output stable and readable directly from the emitter
  across showcase models
- [ ] decide whether public guidance should require `const` codec objects

### Test Work

- [ ] add golden coverage for every rename rule
- [x] add enum SerDe runtime coverage for normal enums, renamed enum values,
  enum collections, unknown values, and codec-backed enhanced enums
- [x] add exact-output coverage for generated sealed variant classes and
  constructor edge cases
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
- [x] runtime tests for enum fields, enum collections, declaration-level enum
  rename rules, unknown enum values, and codec-backed enhanced enum fields
- [x] runtime tests for tagged and untagged sealed class variants
- [x] dedicated malformed-input tests with key-aware decode diagnostics
- [x] negative tests for unsupported function serialization
- [ ] negative tests for unsupported record serialization

## Release Criteria

SerDe is in a good release state when:

- [x] common REST-style models generate cleanly without manual patching
- [x] generated code stays stable without any formatter post-process
- [x] diagnostics are deterministic and actionable for implemented SerDe paths
- [x] public annotation APIs have Dartdoc
- [x] the product showcase covers real nested, enum, and codec-backed models
