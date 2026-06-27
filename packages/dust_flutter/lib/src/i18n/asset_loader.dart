import 'dart:convert';

import 'package:flutter/services.dart';

import 'i18n.dart';

/// Default ARB asset pattern used by generated i18n bootstrap code.
const defaultI18nAssetPattern = 'assets/i18n/{locale}/{namespace}.arb';

/// Adds ARB asset loading helpers to [I18nController].
extension I18nAssetLoading on I18nController {
  /// Loads one ARB asset into this controller.
  Future<I18nBundle> loadAssetBundle({
    required String locale,
    required String namespace,
    AssetBundle? assetBundle,
    String assetPattern = defaultI18nAssetPattern,
  }) async {
    final source = await (assetBundle ?? rootBundle).loadString(
      _assetPath(assetPattern, locale: locale, namespace: namespace),
    );
    final bundle = I18nArbParser.parse(
      source,
      locale: locale,
      namespace: namespace,
    );
    addBundle(bundle);
    return bundle;
  }

  /// Loads multiple ARB assets into this controller and notifies once.
  Future<List<I18nBundle>> loadAssetBundles({
    required Iterable<String> namespaces,
    Iterable<String>? locales,
    AssetBundle? assetBundle,
    String assetPattern = defaultI18nAssetPattern,
  }) async {
    final bundle = assetBundle ?? rootBundle;
    final loaded = <I18nBundle>[];
    for (final locale in locales ?? config.locales) {
      for (final namespace in namespaces) {
        final source = await bundle.loadString(
          _assetPath(assetPattern, locale: locale, namespace: namespace),
        );
        loaded.add(
          I18nArbParser.parse(
            source,
            locale: locale,
            namespace: namespace,
          ),
        );
      }
    }
    addBundles(loaded);
    return loaded;
  }
}

/// Parser for the ARB subset used by the runtime loader.
final class I18nArbParser {
  const I18nArbParser._();

  /// Parses one ARB source into an [I18nBundle].
  static I18nBundle parse(
    String source, {
    required String locale,
    required String namespace,
  }) {
    final decoded = jsonDecode(source);
    if (decoded is! Map<String, Object?>) {
      throw const FormatException('ARB asset must contain a JSON object.');
    }

    final messages = <String, String>{};
    for (final entry in decoded.entries) {
      if (entry.key.startsWith('@')) continue;
      final value = entry.value;
      if (value is String) {
        messages[entry.key] = value;
      }
    }

    return I18nBundle(
      locale: locale,
      namespace: namespace,
      messages: Map<String, String>.unmodifiable(messages),
    );
  }
}

String _assetPath(
  String pattern, {
  required String locale,
  required String namespace,
}) {
  return pattern
      .replaceAll('{locale}', locale)
      .replaceAll('{namespace}', namespace);
}
