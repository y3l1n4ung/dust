/// Runtime JSON conversion helpers used by Dust-generated serde code.
///
/// Generated `.g.dart` files are `part of` the user library, so they reuse the
/// app author's existing `package:dust_dart/serde.dart` import instead of
/// emitting duplicate private helpers into every generated file.
abstract final class JsonHelper {
  /// Throws a consistent type error for one JSON key.
  static Never typeError(Object? value, String key, String expected) {
    throw ArgumentError.value(value, key, 'expected $expected');
  }

  /// Reads [value] as [T], or throws a typed JSON error for [key].
  static T as<T>(Object? value, String key, String expected) {
    return value is T ? value : typeError(value, key, expected);
  }

  /// Parses a string-backed scalar with [parse].
  static T parseString<T>(
    Object? value,
    String key,
    String expected,
    T? Function(String value) parse,
  ) {
    return parse(as<String>(value, key, 'String')) ??
        typeError(value, key, expected);
  }

  /// Reads [value] as a JSON list.
  static List<Object?> asList(Object? value, String key) {
    return as<List>(value, key, 'List<Object?>').cast<Object?>();
  }

  /// Reads [value] as a `Map<String, Object?>`.
  static Map<String, Object?> asMap(Object? value, String key) {
    final map = as<Map>(value, key, 'Map<String, Object?>');
    try {
      return Map<String, Object?>.from(map);
    } on TypeError {
      return typeError(value, key, 'Map<String, Object?>');
    }
  }

  /// Reads an ISO-8601 [DateTime] string.
  static DateTime asDateTime(Object? value, String key) {
    return parseString(
      value,
      key,
      'ISO-8601 DateTime string',
      DateTime.tryParse,
    );
  }

  /// Reads a URI string.
  static Uri asUri(Object? value, String key) {
    return parseString(value, key, 'Uri string', Uri.tryParse);
  }

  /// Reads a [BigInt] string.
  static BigInt asBigInt(Object? value, String key) {
    return parseString(value, key, 'BigInt string', BigInt.tryParse);
  }

  /// Decodes [value] through a user-provided `SerDeCodec`.
  static T decodeWithCodec<T>(dynamic codec, Object? value, String key) {
    if (value == null) {
      throw ArgumentError.value(value, key, 'expected value for SerDeCodec');
    }
    try {
      return codec.deserialize(value as dynamic) as T;
    } catch (error) {
      throw ArgumentError.value(
        value,
        key,
        'failed SerDeCodec decode: $error',
      );
    }
  }
}
