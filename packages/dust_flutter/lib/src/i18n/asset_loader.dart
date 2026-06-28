import 'dart:convert';

import 'package:dust_flutter/src/i18n/i18n.dart';
import 'package:flutter/services.dart';

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
  ///
  /// When [namespaces] is omitted and [I18nConfig.namespaces] is empty,
  /// namespaces are discovered from Flutter's asset manifest.
  Future<List<I18nBundle>> loadAssetBundles({
    Iterable<String>? namespaces,
    Iterable<String>? locales,
    AssetBundle? assetBundle,
    String assetPattern = defaultI18nAssetPattern,
  }) async {
    final bundle = assetBundle ?? rootBundle;
    final selectedLocales = List<String>.of(locales ?? config.locales);
    final configuredNamespaces = namespaces ?? config.namespaces;
    final selectedNamespaces = configuredNamespaces.isEmpty
        ? await _discoverNamespaces(
            bundle,
            locales: selectedLocales,
            assetPattern: assetPattern,
          )
        : List<String>.of(configuredNamespaces);
    if (selectedNamespaces.isEmpty) {
      throw StateError('No i18n namespaces found.');
    }
    final loaded = <I18nBundle>[];
    for (final locale in selectedLocales) {
      for (final namespace in selectedNamespaces) {
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
abstract final class I18nArbParser {
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

Future<List<String>> _discoverNamespaces(
  AssetBundle bundle, {
  required Iterable<String> locales,
  required String assetPattern,
}) async {
  final manifest = await AssetManifest.loadFromAssetBundle(bundle);
  final namespaces = <String>{};
  for (final asset in manifest.listAssets()) {
    final namespace = _namespaceFromAssetPath(
      assetPattern,
      asset,
      locales: locales,
    );
    if (namespace != null) {
      namespaces.add(namespace);
    }
  }
  return (namespaces.toList()..sort());
}

String? _namespaceFromAssetPath(
  String pattern,
  String asset, {
  required Iterable<String> locales,
}) {
  for (final locale in locales) {
    final parts = pattern.replaceAll('{locale}', locale).split('{namespace}');
    if (parts.length != 2) continue;
    final prefix = parts.first;
    final suffix = parts.last;
    if (!asset.startsWith(prefix) || !asset.endsWith(suffix)) continue;
    final namespace =
        asset.substring(prefix.length, asset.length - suffix.length);
    if (namespace.isNotEmpty) return namespace;
  }
  return null;
}
