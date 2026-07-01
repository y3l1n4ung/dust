// coverage:ignore-file

import '../derive/base.dart';

/// Generates `toJson()` support for the annotated class or enum.
final class Serialize extends DeriveTrait {
  /// Creates the `Serialize` derive marker.
  const Serialize();
}

/// Generates `_$TypeFromJson(...)` support for the annotated class or enum.
final class Deserialize extends DeriveTrait {
  /// Creates the `Deserialize` derive marker.
  const Deserialize();
}

/// Field-level conversion contract for custom Dust serde handling.
///
/// Dust owns nullability around the field. Codecs only serialize and
/// deserialize the non-null value itself.
abstract interface class SerDeCodec<DartT, JsonT> {
  /// Creates one codec contract object.
  const SerDeCodec(); // coverage:ignore-line

  /// Converts one Dart value into its JSON representation.
  JsonT serialize(DartT value);

  /// Converts one JSON value into its Dart representation.
  DartT deserialize(JsonT value);
}

/// The rename strategy used when Dust derives JSON keys automatically.
///
/// These values intentionally mirror the common Rust `serde(rename_all = "...")`
/// naming families, but use Dart enum-friendly casing.
enum SerDeRename {
  /// Convert names to `lowercase`.
  lowerCase,

  /// Convert names to `UPPERCASE`.
  upperCase,

  /// Convert names to `PascalCase`.
  pascalCase,

  /// Convert names to `camelCase`.
  camelCase,

  /// Convert names to `snake_case`.
  snakeCase,

  /// Convert names to `SCREAMING_SNAKE_CASE`.
  screamingSnakeCase,

  /// Convert names to `kebab-case`.
  kebabCase,

  /// Convert names to `SCREAMING-KEBAB-CASE`.
  screamingKebabCase,
}

/// Configures how Dust generates or interprets serde metadata.
///
/// This is intentionally modeled after Rust's `#[serde(...)]` attribute:
/// the same annotation can be placed on a declaration or on a field.
///
/// Common declaration-level options:
/// - [renameAll]
/// - [tag]
/// - [content]
/// - [untagged]
/// - [disallowUnrecognizedKeys]
///
/// Common field-level options:
/// - [rename]
/// - [defaultValue]
/// - [skip]
/// - [skipSerializing]
/// - [skipDeserializing]
/// - [aliases]
/// - [using]
final class SerDe extends DeriveConfig {
  /// Explicit serde rename for the annotated declaration or field.
  final String? rename;

  /// Automatic rename rule applied to child fields or enum variants.
  final SerDeRename? renameAll;

  /// JSON discriminator field for sealed class variants.
  ///
  /// On a sealed class, redirecting factory constructors define the variants;
  /// Dust generates omitted concrete target classes into the `.g.dart` part.
  final String? tag;

  /// JSON payload field for adjacent-tagged sealed class variants.
  final String? content;

  /// Whether sealed class deserialization should try variants without a tag.
  final bool untagged;

  /// Fallback value used when deserialization omits the annotated field.
  ///
  /// Fields skipped for deserialization with [skip] or [skipDeserializing]
  /// must provide this value so generated constructors stay total.
  final Object? defaultValue;

  /// Whether the field should be skipped for both serialization and
  /// deserialization.
  ///
  /// When [disallowUnrecognizedKeys] is enabled, skipped fields are not
  /// accepted as input keys.
  final bool skip;

  /// Whether the field should be skipped only for serialization.
  ///
  /// The field is still read during deserialization.
  final bool skipSerializing;

  /// Whether the field should be skipped only for deserialization.
  ///
  /// When [disallowUnrecognizedKeys] is enabled, this field is not accepted as
  /// an input key.
  final bool skipDeserializing;

  /// Alternate accepted input names for one field during deserialization.
  final List<String> aliases;

  /// Custom field codec used instead of Dust's built-in serde mapping.
  ///
  /// This should usually be a const object that implements [SerDeCodec]. Dust
  /// handles nullable fields outside the codec; codecs convert non-null values.
  final Object? using;

  /// Whether generated deserialization should reject unknown JSON keys on the
  /// annotated declaration.
  ///
  /// Primary field keys and [aliases] are recognized. Fields skipped for
  /// deserialization with [skip] or [skipDeserializing] are not accepted as
  /// input keys.
  final bool disallowUnrecognizedKeys;

  /// Creates one serde configuration annotation.
  const SerDe({
    this.rename,
    this.renameAll,
    this.tag,
    this.content,
    this.untagged = false,
    this.defaultValue,
    this.skip = false,
    this.skipSerializing = false,
    this.skipDeserializing = false,
    this.aliases = const [],
    this.using,
    this.disallowUnrecognizedKeys = false,
  });
}
