# dust_plugin_serde

The built-in Dust plugin for generating JSON serialization and deserialization code.

## Features

- **Class SerDe**: Generates `toJson()` and `_$ClassNameFromJson()` for Dart classes.
- **Enum SerDe**: Generates `_$EnumNameToJson()` and `_$EnumNameFromJson()` for Dart enums (serialized as strings).
- **Nested Models**: Automatically handles nesting of other classes/enums that also use `Serialize` or `Deserialize`.
- **Case Conversion**: Supports various renaming rules via `@SerDe(renameAll: ...)`.
- **Field Aliases**: Supports multiple incoming JSON keys for a single field via `@SerDe(aliases: [...])`.
- **Custom Codecs**: Use custom logic for specific fields via `@SerDe(using: ...)`.
- **Default Values**: Provides fallback values when a JSON key is missing.
- **Validation**: Ensures classes have valid constructors and supported field types for deserialization.

## Usage

Annotate your Dart declarations with `@Derive([Serialize(), Deserialize()])`:

```dart
@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase)
class User {
  final String fullName;
  
  @SerDe(rename: 'user_age')
  final int age;

  const User(this.fullName, this.age);
}
```

## Renaming Rules

Supported `SerDeRename` values:
- `none` (default)
- `camelCase`
- `pascalCase`
- `snakeCase`
- `screamingSnakeCase`
- `kebabCase`
- `screamingKebabCase`
- `lowerCase`
- `upperCase`

## Implementation Details

- **Serialization**: Generated as a mixin member `toJson()` which calls a private top-level helper `_$ClassNameToJson()`.
- **Deserialization**: Generated as a private top-level helper `_$ClassNameFromJson()`. You can hook this into your class factory: `factory ClassName.fromJson(Map<String, dynamic> json) => _$ClassNameFromJson(json);`.
- **Validation**: The plugin validates that all fields can be initialized by at least one constructor before generating code.
