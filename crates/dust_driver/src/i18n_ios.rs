use std::{fs, path::Path};

use dust_diagnostics::Diagnostic;
use dust_workspace::{I18nConfig, I18nIosConfig};

/// Result of reconciling one native iOS metadata file.
pub(crate) struct IosPlistStatus {
    /// Path of the inspected plist.
    pub(crate) path: std::path::PathBuf,
    /// Whether the desired content differs from disk.
    pub(crate) changed: bool,
}

/// Reconciles configured locales into an opt-in iOS Info.plist.
pub(crate) fn sync_ios_info_plist(
    package_root: &Path,
    config: &I18nConfig,
    write_output: bool,
) -> Result<Option<IosPlistStatus>, Diagnostic> {
    let Some(ios) = &config.ios else {
        return Ok(None);
    };
    reconcile_plist(
        package_root,
        ios,
        &config.locales,
        config.fallback_locale(),
        write_output,
    )
}

/// Checks that an opt-in iOS Info.plist is synchronized with configured locales.
pub(crate) fn check_ios_info_plist(
    package_root: &Path,
    config: &I18nConfig,
) -> Result<Option<IosPlistStatus>, Diagnostic> {
    sync_ios_info_plist(package_root, config, false)
}

/// Maps Dust's underscore/dash locale syntax to an Apple localization tag.
pub(crate) fn ios_locale_identifier(locale: &str) -> String {
    let mut parts = locale.split(['_', '-']);
    let language = parts.next().unwrap_or(locale).to_ascii_lowercase();
    let mut output = vec![language];
    for part in parts {
        if part.is_empty() {
            continue;
        }
        if is_script(part) {
            let mut chars = part.chars();
            let value = chars
                .next()
                .map(|first| {
                    first.to_ascii_uppercase().to_string() + &chars.as_str().to_ascii_lowercase()
                })
                .unwrap_or_default();
            output.push(value);
        } else {
            output.push(part.to_ascii_uppercase());
        }
    }
    output.join("-")
}

/// Reconciles the plist text while leaving unrelated keys and values intact.
fn reconcile_plist(
    package_root: &Path,
    config: &I18nIosConfig,
    locales: &[String],
    fallback_locale: &str,
    write_output: bool,
) -> Result<Option<IosPlistStatus>, Diagnostic> {
    let path = package_root.join(&config.info_plist);
    let source = fs::read_to_string(&path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read iOS Info.plist `{}`: {error}",
            path.display()
        ))
    })?;
    let desired_locales = locales
        .iter()
        .map(|locale| ios_locale_identifier(locale))
        .collect::<Vec<_>>();
    let mut updated = replace_localizations(&source, &desired_locales)?;
    if config.sync_development_region {
        updated = replace_development_region(&updated, &ios_locale_identifier(fallback_locale))?;
    }
    let changed = updated != source;
    if write_output && changed {
        fs::write(&path, &updated).map_err(|error| {
            Diagnostic::error(format!(
                "failed to write iOS Info.plist `{}`: {error}",
                path.display()
            ))
        })?;
    }
    Ok(Some(IosPlistStatus { path, changed }))
}

/// Replaces or inserts the CFBundleLocalizations array.
fn replace_localizations(source: &str, locales: &[String]) -> Result<String, Diagnostic> {
    let Some(key_start) = source.find("<key>CFBundleLocalizations</key>") else {
        return insert_localizations(source, locales);
    };
    let array_start = source[key_start..]
        .find("<array>")
        .map(|offset| key_start + offset)
        .ok_or_else(|| Diagnostic::error("CFBundleLocalizations must be an array in Info.plist"))?;
    let body_start = array_start + "<array>".len();
    let body = render_array_body(source, array_start, locales);
    let close_start = source[body_start..]
        .find("</array>")
        .map(|offset| body_start + offset)
        .ok_or_else(|| {
            Diagnostic::error("CFBundleLocalizations array is not closed in Info.plist")
        })?;
    Ok(format!(
        "{}{}{}{}",
        &source[..body_start],
        body,
        &source[close_start..close_start + "</array>".len()],
        &source[close_start + "</array>".len()..]
    ))
}

/// Inserts a CFBundleLocalizations array before the root dictionary close.
fn insert_localizations(source: &str, locales: &[String]) -> Result<String, Diagnostic> {
    let close_start = source
        .rfind("</dict>")
        .ok_or_else(|| Diagnostic::error("Info.plist is missing its root dict"))?;
    let key_indent = source[..close_start]
        .rsplit_once('\n')
        .map(|(_, line)| {
            line.chars()
                .take_while(|value| value.is_whitespace())
                .collect::<String>()
        })
        .unwrap_or_else(|| "\t".to_owned());
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let body = render_array_body_with_indent(&key_indent, newline, locales);
    let block = format!(
        "{newline}{indent}<key>CFBundleLocalizations</key>{newline}{indent}<array>{body}</array>",
        indent = key_indent
    );
    Ok(format!(
        "{}{}{}",
        &source[..close_start],
        block,
        &source[close_start..]
    ))
}

/// Replaces or inserts the development region when explicitly enabled.
fn replace_development_region(source: &str, fallback: &str) -> Result<String, Diagnostic> {
    let Some(key_start) = source.find("<key>CFBundleDevelopmentRegion</key>") else {
        let body = format!("\n\t<string>{fallback}</string>");
        let close_start = source
            .rfind("</dict>")
            .ok_or_else(|| Diagnostic::error("Info.plist is missing its root dict"))?;
        return Ok(format!(
            "{}\n\t<key>CFBundleDevelopmentRegion</key>{}\n\t{}",
            &source[..close_start],
            body,
            &source[close_start..]
        ));
    };
    let value_start = source[key_start..]
        .find("<string>")
        .map(|offset| key_start + offset + "<string>".len())
        .ok_or_else(|| {
            Diagnostic::error("CFBundleDevelopmentRegion must be a string in Info.plist")
        })?;
    let value_end = source[value_start..]
        .find("</string>")
        .map(|offset| value_start + offset)
        .ok_or_else(|| Diagnostic::error("CFBundleDevelopmentRegion string is not closed"))?;
    Ok(format!(
        "{}{}{}",
        &source[..value_start],
        fallback,
        &source[value_end..]
    ))
}

/// Renders array entries using the existing plist indentation style.
fn render_array_body(source: &str, array_start: usize, locales: &[String]) -> String {
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let array_indent = source[..array_start]
        .rsplit_once('\n')
        .map(|(_, line)| {
            line.chars()
                .take_while(|value| value.is_whitespace())
                .collect::<String>()
        })
        .unwrap_or_else(|| "\t".to_owned());
    render_array_body_with_indent(&array_indent, newline, locales)
}

/// Renders array entries with a supplied array indentation.
fn render_array_body_with_indent(array_indent: &str, newline: &str, locales: &[String]) -> String {
    let string_indent = format!("{array_indent}\t");
    let entries = locales
        .iter()
        .map(|locale| format!("{newline}{string_indent}<string>{locale}</string>"))
        .collect::<String>();
    format!("{entries}{newline}{array_indent}")
}

/// Returns whether a locale subtag is a four-letter script code.
fn is_script(value: &str) -> bool {
    value.len() == 4 && value.chars().all(char::is_alphabetic)
}

#[cfg(test)]
mod tests {
    use super::{ios_locale_identifier, replace_development_region, replace_localizations};

    #[test]
    fn maps_language_script_and_region_tags() {
        assert_eq!(ios_locale_identifier("en"), "en");
        assert_eq!(ios_locale_identifier("my"), "my");
        assert_eq!(ios_locale_identifier("zh_Hans_CN"), "zh-Hans-CN");
        assert_eq!(ios_locale_identifier("en_US"), "en-US");
    }

    #[test]
    fn replaces_only_localization_array_and_is_idempotent() {
        let source = "<dict>\n\t<key>CustomKey</key>\n\t<string>keep</string>\n\t<key>CFBundleLocalizations</key>\n\t<array>\n\t\t<string>old</string>\n\t</array>\n</dict>\n";
        let locales = vec!["en".to_owned(), "zh-Hans-CN".to_owned()];
        let updated = replace_localizations(source, &locales).unwrap();

        assert!(updated.contains("<key>CustomKey</key>"));
        assert!(updated.contains("<string>keep</string>"));
        assert!(!updated.contains("<string>old</string>"));
        assert_eq!(replace_localizations(&updated, &locales).unwrap(), updated);
    }

    #[test]
    fn updates_development_region_value_without_touching_other_strings() {
        let source = "<dict>\n\t<key>CFBundleDevelopmentRegion</key>\n\t<string>$(DEVELOPMENT_LANGUAGE)</string>\n\t<key>Title</key>\n\t<string>Fixture</string>\n</dict>\n";
        let updated = replace_development_region(source, "en").unwrap();

        assert!(updated.contains("<string>en</string>"));
        assert!(updated.contains("<string>Fixture</string>"));
    }
}
