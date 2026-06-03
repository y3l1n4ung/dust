import 'dart:convert';

/// Decodes a JSON object stored in a text column.
Map<String, Object?> decodeJsonObject(String source) {
  final decoded = jsonDecode(source);
  if (decoded is! Map<String, Object?>) {
    throw FormatException('Expected a JSON object column value.', source);
  }
  return decoded;
}
