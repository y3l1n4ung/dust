# Serde Guide

Use `derive_serde_annotation` when you want Dust to generate JSON
serialization and deserialization.

This package re-exports `derive_annotation`, so one import covers both the core
derive traits and the serde traits.

## Install

Add the package to `pubspec.yaml`:

```yaml
dependencies:
  derive_serde_annotation: ^0.1.0
```

Then fetch packages:

```bash
dart pub get
```

## Basic Example

Reference file:

- [examples/product_showcase/lib/models/json_profile.dart](../../examples/product_showcase/lib/models/json_profile.dart)

```dart
import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'json_profile.g.dart';

@Derive([ToString(), Eq(), Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)
class JsonProfile with _$JsonProfileDust {
  const JsonProfile({
    required this.id,
    this.displayName,
    this.tags = const ['guest'],
  });

  factory JsonProfile.fromJson(Map<String, Object?> json) =>
      _$JsonProfileFromJson(json);

  final String id;

  @SerDe(rename: 'display_name', aliases: ['displayName'])
  final String? displayName;

  @SerDe(defaultValue: ['guest'])
  final List<String> tags;
}
```

Run generation:

```bash
dust build
```

## What Gets Generated

`Serialize()` generates:

- `toJson()`

`Deserialize()` generates:

- `_$TypeFromJson(...)`

Your class provides the forwarding factory:

```dart
factory JsonProfile.fromJson(Map<String, Object?> json) =>
    _$JsonProfileFromJson(json);
```

## Common Serde Features

Class-level configuration:

- `renameAll`
- `disallowUnrecognizedKeys`

Field-level configuration:

- `rename`
- `aliases`
- `defaultValue`
- `using`

These options let you keep stable Dart field names while handling API-specific
JSON shapes.

## Custom Codec Example

Reference file:

- [examples/product_showcase/lib/models/json_codec_bundle.dart](../../examples/product_showcase/lib/models/json_codec_bundle.dart)

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

@Derive([ToString(), Eq(), Serialize(), Deserialize()])
class JsonCodecBundle with _$JsonCodecBundleDust {
  const JsonCodecBundle({required this.createdAt, this.updatedAt});

  factory JsonCodecBundle.fromJson(Map<String, Object?> json) =>
      _$JsonCodecBundleFromJson(json);

  @SerDe(using: unixEpochDateTimeCodec)
  final DateTime createdAt;

  @SerDe(using: unixEpochDateTimeCodec)
  final DateTime? updatedAt;
}
```

Use a codec when the wire format and the Dart type should stay separate.

## When To Use Serde

Use serde when:

- your models cross a JSON boundary
- you need rename rules or decode defaults
- you need strict validation for unknown keys
- you need custom field-level codecs

If you want generated HTTP clients on top of these models, continue with the
[HttpClient guide](./http.md).

## See Also

- [Package README](../../packages/derive_serde_annotation/README.md)
- [Usage overview](./README.md)
