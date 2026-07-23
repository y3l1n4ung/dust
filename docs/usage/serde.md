# JSON Serialization

Dust generates typed JSON encoding and decoding for Dart classes and enums.

## Rust Serde Inspiration

Dust's JSON API is inspired by [Rust Serde](https://serde.rs/derive.html).
Both systems let a type opt into separate serialization and deserialization,
then customize declarations, variants, and fields with
[attributes](https://serde.rs/attributes.html).

Dust adapts familiar concepts such as rename rules, aliases, defaults, skipped
fields, strict keys, custom codecs, and
[tagged enum representations](https://serde.rs/enum-representations.html) to
Dart annotations and generated `.g.dart` files.

## Add the Package

Install the Dust CLI from the [root guide](../../README.md#installation), then
add the Dart runtime package:

```bash
dart pub add dust_dart
```

> [!TIP]
> The focused `package:dust_dart/serde.dart` import also exports `@Derive` and
> the core derive traits, so one import is enough for data and JSON traits.

## Quick Start

Add `Serialize()` and `Deserialize()` to the traits needed by the model:

```dart
import 'package:dust_dart/serde.dart';

part 'user_profile.g.dart';

@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase)
class UserProfile with _$UserProfile {
  const UserProfile({required this.id, this.displayName});

  factory UserProfile.fromJson(Map<String, Object?> json) =>
      _$UserProfileFromJson(json);

  final String id;
  final String? displayName;
}
```

Generate the part file:

```bash
dust build
```

Use the generated APIs normally:

```dart
final profile = UserProfile.fromJson({
  'id': 'u1',
  'display_name': 'Jane',
});

final json = profile.serialize();
final dartJson = profile.toJson(); // Dart ecosystem mirror.
```

`Serialize()` generates `serialize()` as the Dust source API and `toJson()` as
the Dart ecosystem mirror. `Deserialize()` generates the private
`_$TypeDeserialize(...)` source helper and `_$TypeFromJson(...)` mirror helper;
a class must expose a forwarding `fromJson` factory as shown above. The two
traits can be used independently.

Dust also generates reusable directional objects:

```dart
const serializer = $UserProfileSerializer();
final json = serializer.serialize(profile);

const deserializer = $UserProfileDeserializer();
final profileAgain = deserializer.deserialize(json);
```

These implement `Serializer<UserProfile, Map<String, Object?>>` and
`Deserializer<UserProfile, Map<String, Object?>>`. They are useful when a
framework, registry, or helper wants conversion behavior as a value. Those
contracts require only the Dust source verbs; the `serde.dart` import provides
`toJson` and `fromJson` extension mirrors for Dart ecosystem naming.

> [!IMPORTANT]
> Generation requires the matching `part` directive and generated mixin.
> Deserialization also requires the forwarding `fromJson` factory shown above.

## Configuration

Apply `@SerDe` to a class, enum, enum variant, or field.

### Declaration and Variant Options

| Option | Behavior |
| :--- | :--- |
| `rename` | Sets one explicit JSON name. |
| `renameAll` | Applies a naming strategy to fields or enum variants. |
| `disallowUnrecognizedKeys` | Rejects input keys that do not map to a deserialized field or alias. |
| `tag` | Sets the discriminator key for sealed variants. |
| `content` | Adds a payload key for adjacent tagging. |
| `untagged` | Tries sealed variants in declaration order without a discriminator. |

### Field Options

| Option | Behavior |
| :--- | :--- |
| `rename` | Sets the primary JSON key. |
| `aliases` | Accepts additional keys during decoding. |
| `defaultValue` | Supplies a value when the input key is absent. |
| `skip` | Omits the field in both directions. |
| `skipSerializing` | Omits the field from `serialize()` and `toJson()`. |
| `skipDeserializing` | Omits the field from `_$TypeDeserialize(...)` and `fromJson()`. |
| `using` | Uses a `SerDeCodec` for the field. |

> [!IMPORTANT]
> Fields skipped during deserialization must have a `defaultValue` so Dust can
> still call the model constructor. In strict-key mode, skipped input keys are
> rejected rather than silently ignored.

## Rename Strategies

| Strategy | `createdAt` becomes |
| :--- | :--- |
| `lowerCase` | `createdat` |
| `upperCase` | `CREATEDAT` |
| `pascalCase` | `CreatedAt` |
| `camelCase` | `createdAt` |
| `snakeCase` | `created_at` |
| `screamingSnakeCase` | `CREATED_AT` |
| `kebabCase` | `created-at` |
| `screamingKebabCase` | `CREATED-AT` |

## Supported Types

Dust provides built-in mapping for:

- `String`, `int`, `bool`, `double`, `num`, `Object`, and `dynamic`
- `DateTime`, `Uri`, and `BigInt`
- nullable values
- `List<T>`, `Set<T>`, and `Map<String, T>`
- Dust SerDe classes and enums
- models with compatible `toJson()` or `fromJson(Map<String, Object?>)` APIs
- fields with a custom `SerDeCodec`

Records, function types, and generic named models such as `Page<User>` require
a custom codec. Dust validates JSON capability for types declared in the
current workspace; external package types are checked later by the Dart
analyzer.

## Enums

Enums can derive both directions and use declaration or variant renames:

```dart
@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.kebabCase)
enum UserRole {
  admin,
  moderator,
  @SerDe(rename: 'root')
  superUser,
  @SerDe(skip: true)
  legacy,
}
```

> [!NOTE]
> A skipped enum value remains valid Dart, but generated JSON helpers reject it
> for both encoding and decoding.

## Sealed Classes

Redirecting factories define sealed variants. `tag` creates an internal
discriminator, `tag` plus `content` creates adjacent tagging, and `untagged`
tries each variant in declaration order.

```dart
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

If a redirect target is not declared in source, Dust generates a final concrete
class in the `.g.dart` file. Define the target class yourself when you need to
control its implementation; it must still satisfy the redirecting factory and
Serde requirements.

## Custom Codecs

Use `SerDeCodec<DartT, JsonT>` when the JSON representation differs from the
Dart type. `JsonT` is the exact intermediate JSON shape used by the field:

```dart
final class UnixEpochCodec implements SerDeCodec<DateTime, int> {
  const UnixEpochCodec();

  @override
  int serialize(DateTime value) => value.millisecondsSinceEpoch;

  @override
  DateTime deserialize(int value) =>
      DateTime.fromMillisecondsSinceEpoch(value, isUtc: true);
}

const unixEpochCodec = UnixEpochCodec();

@SerDe(using: unixEpochCodec)
final DateTime createdAt;
```

> [!TIP]
> Dust handles field nullability outside the codec. The codec converts only the
> non-null value. The `serde.dart` import provides `toJson` and `fromJson`
> extension mirrors, so custom codecs usually implement only `serialize` and
> `deserialize`.

> [!TIP]
> Dust emits typed codec calls. If `@SerDe(using: ...)` points a `DateTime`
> field at a codec for another Dart type, `dart analyze` reports the mismatch
> in the generated file. If the incoming JSON has the wrong shape for the codec,
> Dust wraps the failure with the field key.

## Examples

- [Class options and strict keys](../../examples/product_showcase/lib/models/json_serde_options.dart)
- [Enums](../../examples/product_showcase/lib/models/json_enum_bundle.dart)
- [Sealed classes](../../examples/product_showcase/lib/models/json_payment_event.dart)
- [Custom codecs](../../examples/product_showcase/lib/models/json_codec_bundle.dart)
- [Serde tests](../../examples/product_showcase/test/generated_serde_scalars_test.dart)
