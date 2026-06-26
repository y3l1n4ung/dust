# JSON Serialization (Serde)

Dust provides JSON encoding and decoding through `dust_dart`. It generates type-safe codecs by analyzing your class definitions and applied annotations.

---

## Installation

Add the package to your `pubspec.yaml`:

```yaml
dependencies:
  dust_dart: ^0.1.0
```

> [!TIP]
> `package:dust_dart/serde.dart` re-exports the core derive traits, so you only need one import for both basic traits and JSON features.

---

## Basic Example

To enable JSON support, add `Serialize()` and `Deserialize()` to your `@Derive` list.

```dart
import 'package:dust_dart/serde.dart';

part 'user_profile.g.dart';

@Derive([ToString(), Eq(), Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)
class UserProfile with _$UserProfile {
  const UserProfile({
    required this.id,
    this.displayName,
    this.tags = const ['new-user'],
  });

  factory UserProfile.fromJson(Map<String, Object?> json) =>
      _$UserProfileFromJson(json);

  final String id;

  @SerDe(rename: 'display_name', aliases: ['name', 'handle'])
  final String? displayName;

  @SerDe(defaultValue: ['new-user'])
  final List<String> tags;
}
```

> [!IMPORTANT]
> **Requirements for Generation:**
> 1. You **must** include the `part 'filename.g.dart';` directive.
> 2. You **must** add the `with _$ClassName` mixin to your class.
> 3. For deserialization, you **must** provide a `fromJson` factory that forwards to the generated `_$ClassNameFromJson` helper.

---

## Configuration Reference

The `@SerDe` annotation can be applied to both **classes** and **individual fields**.

### Class-Level Options

| Property | Type | Description |
| :--- | :--- | :--- |
| `renameAll` | `SerDeRename` | Automatically renames all fields (e.g., `snakeCase`, `camelCase`). |
| `disallowUnrecognizedKeys` | `bool` | If `true`, deserialization throws an error if the JSON contains keys not mapped to a field. |
| `tag` | `String` | JSON discriminator key for sealed class variants. |
| `content` | `String` | Optional payload key for adjacent-tagged sealed class variants. |
| `untagged` | `bool` | Try sealed class variants in factory declaration order without a discriminator key. |

### Field-Level Options

| Property | Type | Description |
| :--- | :--- | :--- |
| `rename` | `String` | Manually set the JSON key name for this field. |
| `aliases` | `List<String>` | Additional JSON keys that will be accepted during deserialization. |
| `defaultValue` | `Object` | The value used if the key is missing from the JSON input. |
| `skip` | `bool` | Completely ignores the field for both encoding and decoding. |
| `skipSerializing` | `bool` | Excludes the field from `toJson()`. |
| `skipDeserializing` | `bool` | Excludes the field from `fromJson()`. |
| `using` | `SerDeCodec` | Specifies a custom codec for this field (e.g., for Dates). |

---

## Naming Strategies (`SerDeRename`)

When using `renameAll`, the following strategies are available:

| Strategy | Output Example (`createdAt`) |
| :--- | :--- |
| `lowerCase` | `createdat` |
| `upperCase` | `CREATEDAT` |
| `pascalCase` | `CreatedAt` |
| `camelCase` | `createdAt` |
| `snakeCase` | `created_at` |
| `screamingSnakeCase` | `CREATED_AT` |
| `kebabCase` | `created-at` |
| `screamingKebabCase` | `CREATED-AT` |

---

## Enums

Dust supports full serialization for enums. Rename rules applied at the enum level affect all variants.

```dart
@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.kebabCase)
enum UserRole {
  admin,      // "admin"
  moderator,  // "moderator"
  superUser,  // "super-user"
}
```

> [!NOTE]
> Per-variant renames (e.g. `@SerDe(rename: '...')` on a specific enum value) are not yet supported.

---

## Sealed Classes

Dust supports Rust-style tagged sealed classes through redirecting factories.
If the redirected target class is not written in source, Dust generates it in
the `.g.dart` part.

```dart
import 'package:dust_dart/serde.dart';

part 'payment_event.g.dart';

@Derive([Serialize(), Deserialize()])
@SerDe(tag: 'type', renameAll: SerDeRename.snakeCase)
sealed class PaymentEvent with _$PaymentEvent {
  const PaymentEvent();

  factory PaymentEvent.fromJson(Map<String, Object?> json) =>
      _$PaymentEventFromJson(json);

  @SerDe(rename: 'payment_success')
  factory PaymentEvent.success({
    required String id,
    required int cents,
  }) = PaymentSuccess;

  factory PaymentEvent.failed({
    required String id,
    required String reason,
  }) = PaymentFailed;
}
```

The generated part contains `PaymentSuccess` and `PaymentFailed` as concrete
`final class` variants, plus variant-specific `fromJson` helpers. The base
`toJson()` mixin still handles serialization for all variants.

Use an explicit source class instead of generated variants when the variant
needs extra derives such as `CopyWith()` or `Eq()`, custom methods, custom
interfaces, or nullable named parameters that must still be marked `required`.

For adjacent tagging, add `content`:

```dart
@SerDe(tag: 'kind', content: 'payload', renameAll: SerDeRename.snakeCase)
sealed class PaymentEvent with _$PaymentEvent {
  const PaymentEvent();
}
```

For untagged sealed payloads, set `untagged: true`; decoding tries redirecting
factory variants in declaration order and fails if no variant shape matches.

---

## Custom Codecs

Use a `SerDeCodec` when the JSON representation differs from your Dart type.

```dart
final class DateTimeCodec implements SerDeCodec<DateTime, int> {
  const DateTimeCodec();

  @override
  int serialize(DateTime value) => value.millisecondsSinceEpoch;

  @override
  DateTime deserialize(int value) => DateTime.fromMillisecondsSinceEpoch(value);
}

// Usage
@SerDe(using: DateTimeCodec())
final DateTime createdAt;
```

> [!TIP]
> Custom codecs are ideal for handling legacy API formats or complex object transformations while keeping your data class clean.

---

## Generation Output

Dust generates a private mixin and a helper function. Below is a preview of the generated code structure:

```dart
// user_profile.g.dart (Simplified)
mixin _$UserProfile on UserProfile {
  Map<String, Object?> toJson() => {
    'id': id,
    'display_name': displayName,
    'tags': tags,
  };
}

UserProfile _$UserProfileFromJson(Map<String, Object?> json) => UserProfile(
  id: json['id'] as String,
  displayName: json['display_name'] as String?,
  tags: (json['tags'] as List?)?.cast<String>() ?? const ['new-user'],
);
```

---

## Migration Guide

**Coming from `json_serializable`?**

| Feature | `json_serializable` | Dust |
| :--- | :--- | :--- |
| Annotation | `@JsonSerializable()` | `@Derive([Serialize(), Deserialize()])` |
| Key Rename | `@JsonKey(name: '...')` | `@SerDe(rename: '...')` |
| Defaults | `@JsonKey(defaultValue: ...)` | `@SerDe(defaultValue: ...)` |
| Unknown Keys | `disallowUnrecognizedKeys: true` | `@SerDe(disallowUnrecognizedKeys: true)` |
