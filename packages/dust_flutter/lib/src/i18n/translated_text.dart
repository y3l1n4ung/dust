import 'package:dust_flutter/src/i18n/i18n.dart';
import 'package:flutter/widgets.dart';

/// Text widget that resolves an i18n key at build time.
class TranslatedText extends StatelessWidget {
  /// Creates statically discoverable translated text.
  const TranslatedText(
    this.translationKey, {
    super.key,
    this.defaultText,
    this.args = const {},
    this.style,
    this.strutStyle,
    this.textAlign,
    this.textDirection,
    this.locale,
    this.softWrap,
    this.overflow,
    this.textScaleFactor,
    this.textScaler,
    this.maxLines,
    this.semanticsLabel,
    this.semanticsIdentifier,
    this.textWidthBasis,
    this.textHeightBehavior,
    this.selectionColor,
  }) : fallback = null;

  /// Creates runtime-only translated text for API or JSON driven keys.
  const TranslatedText.dynamic(
    this.translationKey, {
    super.key,
    this.fallback,
    this.args = const {},
    this.style,
    this.strutStyle,
    this.textAlign,
    this.textDirection,
    this.locale,
    this.softWrap,
    this.overflow,
    this.textScaleFactor,
    this.textScaler,
    this.maxLines,
    this.semanticsLabel,
    this.semanticsIdentifier,
    this.textWidthBasis,
    this.textHeightBehavior,
    this.selectionColor,
  }) : defaultText = null;

  /// Translation key resolved through [I18nController.translate].
  final String translationKey;

  /// Default text for static keys when no bundle message exists.
  final String? defaultText;

  /// Runtime fallback for dynamic keys when no bundle message exists.
  final String? fallback;

  /// Placeholder values used to replace `{name}` tokens.
  final Map<String, Object?> args;

  /// Text style forwarded to [Text.style].
  final TextStyle? style;

  /// Strut style forwarded to [Text.strutStyle].
  final StrutStyle? strutStyle;

  /// Text alignment forwarded to [Text.textAlign].
  final TextAlign? textAlign;

  /// Text direction forwarded to [Text.textDirection].
  final TextDirection? textDirection;

  /// Locale forwarded to [Text.locale] for font selection.
  final Locale? locale;

  /// Soft-wrap behavior forwarded to [Text.softWrap].
  final bool? softWrap;

  /// Overflow behavior forwarded to [Text.overflow].
  final TextOverflow? overflow;

  /// Legacy text scale factor converted into a [TextScaler].
  final double? textScaleFactor;

  /// Text scaler forwarded to [Text.textScaler].
  final TextScaler? textScaler;

  /// Maximum line count forwarded to [Text.maxLines].
  final int? maxLines;

  /// Semantics label forwarded to [Text.semanticsLabel].
  final String? semanticsLabel;

  /// Semantics identifier forwarded to [Text.semanticsIdentifier].
  final String? semanticsIdentifier;

  /// Text width basis forwarded to [Text.textWidthBasis].
  final TextWidthBasis? textWidthBasis;

  /// Text height behavior forwarded to [Text.textHeightBehavior].
  final TextHeightBehavior? textHeightBehavior;

  /// Selection color forwarded to [Text.selectionColor].
  final Color? selectionColor;

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
      strutStyle: strutStyle,
      textAlign: textAlign,
      textDirection: textDirection,
      locale: locale,
      softWrap: softWrap,
      overflow: overflow,
      textScaler: textScaler ?? _legacyTextScaler,
      maxLines: maxLines,
      semanticsLabel: semanticsLabel,
      semanticsIdentifier: semanticsIdentifier,
      textWidthBasis: textWidthBasis,
      textHeightBehavior: textHeightBehavior,
      selectionColor: selectionColor,
    );
  }

  TextScaler? get _legacyTextScaler {
    final factor = textScaleFactor;
    return factor == null ? null : TextScaler.linear(factor);
  }
}
