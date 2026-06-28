final RegExp _placeholderPattern = RegExp(r'\{([A-Za-z_][A-Za-z0-9_]*)\}');

/// Replaces `{name}` placeholders in [template] with values from [args].
String interpolateI18n(String template, Map<String, Object?> args) {
  if (args.isEmpty) return template;
  return template.replaceAllMapped(_placeholderPattern, (match) {
    final name = match.group(1)!;
    if (!args.containsKey(name)) return match.group(0)!;
    return args[name]?.toString() ?? '';
  });
}

/// Parsed namespace and local ARB key for one runtime translation key.
final class I18nKeyParts {
  /// Creates parsed key parts.
  const I18nKeyParts(this.namespace, this.localKey);

  /// Splits [key] using the longest configured namespace prefix.
  factory I18nKeyParts.parse(
    String key, {
    Iterable<String> namespaces = const [],
  }) {
    final sortedNamespaces = namespaces.toList()
      ..sort((left, right) => right.length.compareTo(left.length));
    for (final namespace in sortedNamespaces) {
      final prefix = '${namespace}_';
      if (key.startsWith(prefix) && key.length > prefix.length) {
        return I18nKeyParts(namespace, key.substring(prefix.length));
      }
    }

    return I18nKeyParts('', key);
  }

  /// Namespace used to choose the ARB bundle.
  final String namespace;

  /// Message key inside the selected ARB bundle.
  final String localKey;
}
