String shopCategoryKey(String category) {
  return 'shop_category_${shopI18nKeySegment(category)}';
}

String shopI18nKeySegment(String value) {
  final buffer = StringBuffer();
  var previousWasSeparator = false;

  for (final codeUnit in value.toLowerCase().codeUnits) {
    final isDigit = codeUnit >= 48 && codeUnit <= 57;
    final isLetter = codeUnit >= 97 && codeUnit <= 122;
    if (isDigit || isLetter) {
      buffer.writeCharCode(codeUnit);
      previousWasSeparator = false;
    } else if (!previousWasSeparator && buffer.isNotEmpty) {
      buffer.write('_');
      previousWasSeparator = true;
    }
  }

  final normalized = buffer.toString();
  final trimmed = normalized.endsWith('_')
      ? normalized.substring(0, normalized.length - 1)
      : normalized;
  return trimmed.isEmpty ? 'value' : trimmed;
}
