use dust_diagnostics::Diagnostic;
use dust_ir::{
    ConfigApplicationIr, HttpConfigIr, HttpParameterConfigIr, HttpParseThreadIr, MethodIr,
    MethodParamIr, NormalizedConfigIr,
};

use crate::plugin::constants::{
    BODY, EXTRA, FIELD, FORM_URL_ENCODED, HEADER, HEADER_MAP, HEADERS, HTTP_PARSE, MULTI_PART,
    PART, PATH, QUERIES, QUERY,
};
use crate::plugin::model::{HttpTargetMode, HttpVerb, ParseThreadMode, RequestMode};
use crate::plugin::parse::{
    invalid_string_map, parse_config_map_argument, parse_config_string_argument,
};
use crate::plugin::util::{config_name, label};

/// Parsing the threading configuration an HTTP client annotation carries.
mod threading;
use self::threading::*;

/// Reading the verb, path, request mode and parameter sources off one method.
mod methods;
pub(crate) use self::methods::*;

/// Parsed options from the `@HttpClient` annotation.
#[derive(Debug, Clone)]
pub(crate) struct ParsedHttpClientConfig {
    /// Optional default base URL for generated requests.
    pub(crate) base_url: Option<String>,
    /// Target runtime selected for generated imports and helpers.
    pub(crate) target: HttpTargetMode,
    /// Default parse-thread mode for endpoint response decoding.
    pub(crate) parse_thread: ParseThreadMode,
    /// Class-level static headers applied to every endpoint.
    pub(crate) headers: Vec<(String, String)>,
    /// Whether to generate an auxiliary request-mapping test file.
    pub(crate) generate_test: bool,
}

/// Parses class-level `@HttpClient` options.
pub(crate) fn parse_http_client_config(
    config: &ConfigApplicationIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> ParsedHttpClientConfig {
    if let Some(NormalizedConfigIr::Http(HttpConfigIr::Client(normalized))) =
        config.normalized.as_ref()
    {
        return ParsedHttpClientConfig {
            base_url: normalized.base_url.clone(),
            target: match normalized.target {
                dust_ir::HttpTargetIr::Dart => HttpTargetMode::Dart,
                dust_ir::HttpTargetIr::Flutter => HttpTargetMode::Flutter,
            },
            parse_thread: match normalized.parse_thread {
                HttpParseThreadIr::Main => ParseThreadMode::Main,
                HttpParseThreadIr::Isolate => ParseThreadMode::Isolate,
            },
            headers: normalized.headers.clone(),
            generate_test: normalized.generate_test,
        };
    }
    let mut parsed = ParsedHttpClientConfig {
        base_url: None,
        target: HttpTargetMode::Dart,
        parse_thread: ParseThreadMode::Main,
        headers: Vec::new(),
        generate_test: false,
    };

    for (key, _) in config.named_arguments() {
        match key {
            "baseUrl" => match config.named_string("baseUrl") {
                Some(url) => parsed.base_url = Some(url),
                None => diagnostics.push(
                    Diagnostic::error("`HttpClient(baseUrl: ...)` expects a string literal")
                        .with_label(label(config.span, "use a quoted base URL string")),
                ),
            },
            "target" => match config.named_member("target").as_deref() {
                Some("dart") | Some("HttpTarget.dart") => parsed.target = HttpTargetMode::Dart,
                Some("flutter") | Some("HttpTarget.flutter") => {
                    parsed.target = HttpTargetMode::Flutter;
                }
                _ => diagnostics.push(
                    Diagnostic::error(
                        "`HttpClient(target: ...)` must be `HttpTarget.dart` or `HttpTarget.flutter`",
                    )
                    .with_label(label(config.span, "pick one of the supported target enum values")),
                ),
            },
            "parseThread" => parsed.parse_thread = parse_thread_config(config, diagnostics),
            "headers" => parsed.headers = parse_http_client_headers(config, diagnostics),
            "generateTest" => match config.named_bool("generateTest") {
                Some(value) => parsed.generate_test = value,
                None => diagnostics.push(
                    Diagnostic::error("`HttpClient(generateTest: ...)` expects `true` or `false`")
                        .with_label(label(config.span, "use `generateTest: true` to generate request-mapping tests")),
                ),
            },
            other => diagnostics.push(
                Diagnostic::warning(format!("unknown `HttpClient` option `{other}`"))
                    .with_label(label(config.span, "remove or rename this unsupported option")),
            ),
        }
    }

    parsed
}

/// Parses the named `headers:` map from `@HttpClient`.
pub(crate) fn parse_http_client_headers(
    config: &ConfigApplicationIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<(String, String)> {
    if let Some(NormalizedConfigIr::Http(HttpConfigIr::Client(normalized))) =
        config.normalized.as_ref()
    {
        return normalized.headers.clone();
    }
    match config.named_string_map("headers") {
        Some(values) => values,
        None => {
            diagnostics.push(invalid_string_map("HttpClient(headers: ...)", config.span));
            Vec::new()
        }
    }
}

/// Parses a `@Headers` annotation into key/value pairs.
pub(crate) fn parse_headers_config(
    config: &ConfigApplicationIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<(String, String)> {
    if let Some(NormalizedConfigIr::Http(HttpConfigIr::Headers(values))) =
        config.normalized.as_ref()
    {
        return values.clone();
    }
    parse_config_map_argument(config, diagnostics, "Headers")
}

/// Parses all `@Headers` annotations attached to a method.
pub(crate) fn parse_method_headers(
    method: &MethodIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<(String, String)> {
    method
        .configs
        .iter()
        .filter(|config| config_name(&config.symbol.0) == HEADERS)
        .flat_map(|config| parse_headers_config(config, diagnostics))
        .collect()
}

/// Resolves the parse-thread mode for a method after method-level overrides.
pub(crate) fn method_parse_thread(
    method: &MethodIr,
    default: ParseThreadMode,
    diagnostics: &mut Vec<Diagnostic>,
) -> ParseThreadMode {
    for config in &method.configs {
        if config_name(&config.symbol.0) == HTTP_PARSE {
            if let Some(NormalizedConfigIr::Http(HttpConfigIr::Parse(parse))) =
                config.normalized.as_ref()
            {
                return match parse.thread {
                    HttpParseThreadIr::Main => ParseThreadMode::Main,
                    HttpParseThreadIr::Isolate => ParseThreadMode::Isolate,
                };
            }
            return parse_http_parse_config(config, diagnostics);
        }
    }
    default
}

/// Parses an `@HttpParse` annotation.
pub(crate) fn parse_http_parse_config(
    config: &ConfigApplicationIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> ParseThreadMode {
    for (key, _) in config.named_arguments() {
        if key != "thread" {
            diagnostics.push(
                Diagnostic::warning(format!("unknown `HttpParse` option `{key}`")).with_label(
                    label(config.span, "remove or rename this unsupported option"),
                ),
            );
            continue;
        }
        return parse_thread_config(config, diagnostics);
    }
    ParseThreadMode::Main
}
