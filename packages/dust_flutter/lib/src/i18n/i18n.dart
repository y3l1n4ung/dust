import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

/// Runtime i18n configuration.
@immutable
final class I18nConfig {
  /// Creates runtime configuration for the supported [locales].
  const I18nConfig({
    required this.locales,
    required this.fallbackLocale,
  });

  /// Locale codes supported by the application.
  final List<String> locales;

  /// Locale used when the active locale does not contain a key.
  final String fallbackLocale;
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

  /// Namespace, usually the first segment of a key such as `home.title`.
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

    return _interpolate(template, args);
  }

  void _addBundles(Iterable<I18nBundle> bundles) {
    for (final bundle in bundles) {
      _setBundle(bundle);
    }
  }

  void _setBundle(I18nBundle bundle) {
    _checkLocale(bundle.locale);
    _bundles.putIfAbsent(bundle.locale, () => <String, Map<String, String>>{})[
        bundle.namespace] = Map<String, String>.unmodifiable(bundle.messages);
  }

  String? _lookup(String locale, String key) {
    final parts = _I18nKeyParts.parse(key);
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
    super.key,
    required I18nController controller,
    required super.child,
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

/// Text widget that resolves an i18n key at build time.
class TranslatedText extends StatelessWidget {
  /// Creates statically discoverable translated text.
  const TranslatedText(
    this.translationKey, {
    super.key,
    this.defaultText,
    this.args = const {},
    this.style,
    this.textAlign,
    this.overflow,
    this.maxLines,
    this.softWrap,
  }) : fallback = null;

  /// Creates runtime-only translated text for API or JSON driven keys.
  const TranslatedText.dynamic(
    this.translationKey, {
    super.key,
    this.fallback,
    this.args = const {},
    this.style,
    this.textAlign,
    this.overflow,
    this.maxLines,
    this.softWrap,
  }) : defaultText = null;

  /// Translation key.
  final String translationKey;

  /// Default text for static keys.
  final String? defaultText;

  /// Runtime fallback for dynamic keys.
  final String? fallback;

  /// Placeholder values.
  final Map<String, Object?> args;

  /// Text style forwarded to [Text].
  final TextStyle? style;

  /// Text alignment forwarded to [Text].
  final TextAlign? textAlign;

  /// Overflow behavior forwarded to [Text].
  final TextOverflow? overflow;

  /// Maximum line count forwarded to [Text].
  final int? maxLines;

  /// Soft-wrap behavior forwarded to [Text].
  final bool? softWrap;

  @override
  Widget build(BuildContext context) {
    return Text(
      I18nScope.of(context).translate(
        translationKey,
        defaultText: defaultText,
        fallback: fallback,
        args: args,
      ),
      style: style,
      textAlign: textAlign,
      overflow: overflow,
      maxLines: maxLines,
      softWrap: softWrap,
    );
  }
}

final RegExp _placeholderPattern = RegExp(r'\{([A-Za-z_][A-Za-z0-9_]*)\}');

String _interpolate(String template, Map<String, Object?> args) {
  if (args.isEmpty) return template;
  return template.replaceAllMapped(_placeholderPattern, (match) {
    final name = match.group(1)!;
    if (!args.containsKey(name)) return match.group(0)!;
    return args[name]?.toString() ?? '';
  });
}

final class _I18nKeyParts {
  const _I18nKeyParts(this.namespace, this.localKey);

  factory _I18nKeyParts.parse(String key) {
    final index = key.indexOf('.');
    if (index <= 0 || index == key.length - 1) {
      return _I18nKeyParts('', key);
    }
    return _I18nKeyParts(key.substring(0, index), key.substring(index + 1));
  }

  final String namespace;
  final String localKey;
}
