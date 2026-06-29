use std::{
    fs, io,
    path::{Path, PathBuf},
};

use dust_dart_emit::dart_string_literal;
use dust_diagnostics::Diagnostic;
use dust_plugin_api::GENERATED_HEADER;
use dust_workspace::{DustConfig, I18nConfig};

use crate::result::{BuildArtifact, CheckedLibrary};

/// Writes generated i18n bootstrap when configured.
pub(crate) fn build_i18n_bootstrap(
    package_root: &Path,
    config: &DustConfig,
) -> Result<Option<BuildArtifact>, Diagnostic> {
    let Some(status) = emit_i18n_bootstrap(package_root, config, true)? else {
        return Ok(None);
    };
    Ok(Some(BuildArtifact {
        source_path: status.source_path,
        output_path: status.output_path,
        auxiliary_output_paths: Vec::new(),
        changed: status.changed,
        written: status.written,
        cached: false,
        routed: false,
    }))
}

/// Checks generated i18n bootstrap freshness when configured.
pub(crate) fn check_i18n_bootstrap(
    package_root: &Path,
    config: &DustConfig,
) -> Result<Option<CheckedLibrary>, Diagnostic> {
    let Some(status) = emit_i18n_bootstrap(package_root, config, false)? else {
        return Ok(None);
    };
    Ok(Some(CheckedLibrary {
        source_path: status.source_path,
        output_path: status.output_path,
        auxiliary_output_paths: Vec::new(),
        stale: status.changed,
        cached: false,
    }))
}

/// Emits bootstrap source and optionally persists it.
fn emit_i18n_bootstrap(
    package_root: &Path,
    config: &DustConfig,
    write_output: bool,
) -> Result<Option<BootstrapStatus>, Diagnostic> {
    let Some(i18n) = &config.i18n else {
        return Ok(None);
    };

    let output_path = package_root.join("lib/i18n/app_i18n.g.dart");
    let source = render_i18n_bootstrap(i18n);
    let previous = read_existing_output(&output_path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read `{}`: {error}",
            output_path.display()
        ))
    })?;
    let changed = previous.as_deref() != Some(source.as_str());
    let mut written = false;

    if write_output && changed {
        write_output_file(&output_path, &source).map_err(|error| {
            Diagnostic::error(format!(
                "failed to write `{}`: {error}",
                output_path.display()
            ))
        })?;
        written = true;
    }

    Ok(Some(BootstrapStatus {
        source_path: config
            .path
            .clone()
            .unwrap_or_else(|| package_root.join("dust.yaml")),
        output_path,
        changed,
        written,
    }))
}

/// Rendered bootstrap file status.
struct BootstrapStatus {
    /// Source config path.
    source_path: PathBuf,
    /// Generated output path.
    output_path: PathBuf,
    /// Whether generated source differs from disk.
    changed: bool,
    /// Whether generated source was written.
    written: bool,
}

/// Renders `lib/i18n/app_i18n.g.dart`.
fn render_i18n_bootstrap(config: &I18nConfig) -> String {
    let locales = config
        .locales
        .iter()
        .map(|locale| dart_string_literal(locale))
        .collect::<Vec<_>>()
        .join(", ");
    let supported_locales = config
        .locales
        .iter()
        .map(|locale| format!("  {},", render_locale(locale)))
        .collect::<Vec<_>>()
        .join("\n");
    let locale_cases = config
        .locales
        .iter()
        .map(|locale| {
            format!(
                "    case {}:\n      return {};",
                dart_string_literal(locale),
                render_locale(locale)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"{GENERATED_HEADER}
import 'dart:async' show unawaited;

import 'package:dust_flutter/i18n.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter/widgets.dart';

const List<String> appI18nLocales = <String>[{locales}];
const List<Locale> appI18nSupportedLocales = <Locale>[
{supported_locales}
];
const List<LocalizationsDelegate<dynamic>> appI18nLocalizationsDelegates =
    <LocalizationsDelegate<dynamic>>[
  GlobalMaterialLocalizations.delegate,
  GlobalCupertinoLocalizations.delegate,
  GlobalWidgetsLocalizations.delegate,
];
const String appI18nFallbackLocale = {fallback};
const String appI18nAssetPattern = defaultI18nAssetPattern;

Locale appI18nLocaleOf(String locale) {{
  switch (locale) {{
{locale_cases}
    default:
      return Locale(locale);
  }}
}}

const I18nConfig appI18nConfig = I18nConfig(
  locales: appI18nLocales,
  fallbackLocale: appI18nFallbackLocale,
);

class AppI18n extends StatefulWidget {{
  const AppI18n({{
    required this.child,
    this.assetBundle,
    super.key,
  }});

  final Widget child;
  final AssetBundle? assetBundle;

  @override
  State<AppI18n> createState() => _AppI18nState();
}}

class _AppI18nState extends State<AppI18n> {{
  late final I18nController _controller =
      I18nController(config: appI18nConfig);

  @override
  void initState() {{
    super.initState();
    unawaited(_loadBundles());
  }}

  @override
  void dispose() {{
    _controller.dispose();
    super.dispose();
  }}

  @override
  Widget build(BuildContext context) {{
    return I18nScope(
      controller: _controller,
      child: widget.child,
    );
  }}

  Future<void> _loadBundles() async {{
    try {{
      await _controller.loadAssetBundles(
        assetBundle: widget.assetBundle,
        assetPattern: appI18nAssetPattern,
      );
    }} catch (error, stackTrace) {{
      FlutterError.reportError(
        FlutterErrorDetails(
          exception: error,
          stack: stackTrace,
          library: 'dust_flutter',
          context: ErrorDescription('while loading i18n assets'),
        ),
      );
    }}
  }}
}}
"#,
        fallback = dart_string_literal(config.fallback_locale())
    )
}

/// Renders one configured locale as a const Flutter `Locale`.
fn render_locale(locale: &str) -> String {
    let parts = locale.split(['_', '-']).collect::<Vec<_>>();
    let language = parts.first().copied().unwrap_or(locale);
    let mut script = None;
    let mut country = None;
    for part in parts.iter().skip(1) {
        if script.is_none() && is_script_subtag(part) {
            script = Some(*part);
        } else if country.is_none() && is_region_subtag(part) {
            country = Some(*part);
        }
    }
    let mut args = vec![format!("languageCode: {}", dart_string_literal(language))];
    if let Some(script) = script {
        args.push(format!("scriptCode: {}", dart_string_literal(script)));
    }
    if let Some(country) = country {
        args.push(format!("countryCode: {}", dart_string_literal(country)));
    }
    format!("Locale.fromSubtags({})", args.join(", "))
}

/// Returns whether one locale subtag is a Unicode script code.
fn is_script_subtag(value: &str) -> bool {
    value.len() == 4 && value.chars().all(char::is_alphabetic)
}

/// Returns whether one locale subtag is a Unicode region code.
fn is_region_subtag(value: &str) -> bool {
    (value.len() == 2 && value.chars().all(char::is_alphabetic))
        || (value.len() == 3 && value.chars().all(|item| item.is_ascii_digit()))
}

/// Reads an existing generated output file.
fn read_existing_output(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(source)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// Writes generated output, creating parent directories on demand.
fn write_output_file(path: &Path, source: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, source)
}
