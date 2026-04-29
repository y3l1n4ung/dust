import 'package:derive_annotation/derive_annotation.dart';

/// Generates `toJson()` support for the annotated declaration.
final class Serialize extends DeriveTrait {
  /// Creates the `Serialize` derive marker.
  const Serialize();
}

/// Generates `fromJson(...)` support for the annotated declaration.
final class Deserialize extends DeriveTrait {
  /// Creates the `Deserialize` derive marker.
  const Deserialize();
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
/// - [denyUnknownFields]
///
/// Common field-level options:
/// - [rename]
/// - [defaultValue]
/// - [skip]
/// - [skipSerializing]
/// - [skipDeserializing]
/// - [aliases]
final class SerDe extends DeriveConfig {
  /// Explicit serde rename for the annotated declaration or field.
  final String? rename;

  /// Automatic rename rule applied to child fields without an explicit [rename].
  final SerDeRename? renameAll;

  /// Fallback value used when deserialization omits the annotated field.
  final Object? defaultValue;

  /// Whether the field should be skipped for both serialization and
  /// deserialization.
  final bool skip;

  /// Whether the field should be skipped only for serialization.
  final bool skipSerializing;

  /// Whether the field should be skipped only for deserialization.
  final bool skipDeserializing;

  /// Alternate accepted input names for one field during deserialization.
  final List<String> aliases;

  /// Whether generated deserialization should reject unknown JSON keys on the
  /// annotated declaration.
  final bool disallowUnrecognizedKeys;

  /// Creates one serde configuration annotation.
  const SerDe({
    this.rename,
    this.renameAll,
    this.defaultValue,
    this.skip = false,
    this.skipSerializing = false,
    this.skipDeserializing = false,
    this.aliases = const [],
    this.disallowUnrecognizedKeys = false,
  });
}
