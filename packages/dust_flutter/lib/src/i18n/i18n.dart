import 'package:dust_flutter/src/i18n/key.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

/// Runtime i18n configuration.
@immutable
final class I18nConfig {
  /// Creates runtime configuration for the supported [locales].
  const I18nConfig({
    required this.locales,
    required this.fallbackLocale,
    this.namespaces = const [],
  });

  /// Locale codes supported by the application.
  final List<String> locales;

  /// Locale used when the active locale does not contain a key.
  final String fallbackLocale;

  /// Optional translation namespaces with one ARB file per locale.
  ///
  /// When empty, lookups infer namespaces from loaded bundles.
  final List<String> namespaces;
}

/// One loaded namespace of messages for one locale.
@immutable
final class I18nBundle {
  /// Creates a translation bundle.
  const I18nBundle({
    required this.locale,
    required this.namespace,
    required this.messages,
  });

  /// Locale code this bundle belongs to.
  final String locale;

  /// Namespace, usually the first key prefix such as `home` in `home_title`.
  final String namespace;

  /// Translation messages in this namespace.
  final Map<String, String> messages;
}

/// Coordinates locale, bundles, overrides, and translation lookup.
class I18nController extends ChangeNotifier {
  /// Creates an i18n controller with optional in-memory [bundles].
  I18nController({
    required this.config,
    String? locale,
    Iterable<I18nBundle> bundles = const [],
  }) : _locale = locale ?? config.fallbackLocale {
    _checkLocale(_locale);
    _checkLocale(config.fallbackLocale);
    _addBundles(bundles);
  }

  /// Runtime configuration.
  final I18nConfig config;

  final Map<String, Map<String, Map<String, String>>> _bundles =
      <String, Map<String, Map<String, String>>>{};
  final Map<String, String> _overrides = <String, String>{};
  String _locale;

  /// Current locale code.
  String get locale => _locale;

  /// Changes the current locale and notifies listeners.
  void setLocale(String locale) {
    _checkLocale(locale);
    if (_locale == locale) return;
    _locale = locale;
    notifyListeners();
  }

  /// Adds or replaces one translation bundle.
  void addBundle(I18nBundle bundle) {
    _setBundle(bundle);
    notifyListeners();
  }

  /// Adds or replaces multiple translation bundles.
  void addBundles(Iterable<I18nBundle> bundles) {
    var changed = false;
    for (final bundle in bundles) {
      _setBundle(bundle);
      changed = true;
    }
    if (changed) notifyListeners();
  }

  /// Sets or removes one translation override.
  void setOverride(String key, String? value) {
    final previous = _overrides[key];
    if (value == null) {
      if (!_overrides.containsKey(key)) return;
      _overrides.remove(key);
      notifyListeners();
      return;
    }
    if (previous == value) return;
    _overrides[key] = value;
    notifyListeners();
  }

  /// Replaces all runtime translation overrides.
  void setOverrides(Map<String, String> overrides) {
    if (mapEquals(_overrides, overrides)) return;
    _overrides
      ..clear()
      ..addAll(overrides);
    notifyListeners();
  }

  /// Clears all runtime translation overrides.
  void clearOverrides() {
    if (_overrides.isEmpty) return;
    _overrides.clear();
    notifyListeners();
  }

  /// Resolves one translated message.
  String translate(
    String key, {
    String? defaultText,
    String? fallback,
    Map<String, Object?> args = const {},
  }) {
    final template = _overrides[key] ??
        _lookup(_locale, key) ??
        (_locale == config.fallbackLocale
            ? null
            : _lookup(config.fallbackLocale, key)) ??
        defaultText ??
        fallback ??
        key;

    return interpolateI18n(template, args);
  }

  void _addBundles(Iterable<I18nBundle> bundles) {
    bundles.forEach(_setBundle);
  }

  void _setBundle(I18nBundle bundle) {
    _checkLocale(bundle.locale);
    _bundles.putIfAbsent(bundle.locale, () => <String, Map<String, String>>{})[
        bundle.namespace] = Map<String, String>.unmodifiable(bundle.messages);
  }

  String? _lookup(String locale, String key) {
    final namespaces = <String>{
      ...config.namespaces,
      ...?_bundles[locale]?.keys,
    };
    final parts = I18nKeyParts.parse(key, namespaces: namespaces);
    final messages = _bundles[locale]?[parts.namespace];
    return messages?[key] ?? messages?[parts.localKey];
  }

  void _checkLocale(String locale) {
    if (!config.locales.contains(locale)) {
      throw ArgumentError.value(locale, 'locale', 'unsupported locale');
    }
  }
}

/// Provides an [I18nController] to a widget subtree.
class I18nScope extends InheritedNotifier<I18nController> {
  /// Creates an i18n scope.
  const I18nScope({
    required I18nController controller,
    required super.child,
    super.key,
  }) : super(notifier: controller);

  /// Returns the nearest controller, or null when no scope exists.
  static I18nController? maybeOf(BuildContext context) {
    return context.dependOnInheritedWidgetOfExactType<I18nScope>()?.notifier;
  }

  /// Returns the nearest controller.
  static I18nController of(BuildContext context) {
    final controller = maybeOf(context);
    if (controller == null) {
      throw FlutterError('No I18nScope found in context.');
    }
    return controller;
  }
}

/// Translation helpers for [BuildContext].
extension I18nBuildContext on BuildContext {
  /// Translates [key] with optional [defaultText] and placeholder [args].
  String tr(
    String key, {
    String? defaultText,
    Map<String, Object?> args = const {},
  }) {
    return I18nScope.of(this).translate(
      key,
      defaultText: defaultText,
      args: args,
    );
  }
}
