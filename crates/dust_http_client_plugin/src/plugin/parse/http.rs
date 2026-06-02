use dust_diagnostics::Diagnostic;
use dust_ir::{ConfigApplicationIr, MethodIr, MethodParamIr};

use crate::plugin::constants::{
    BODY, EXTRA, FIELD, FORM_URL_ENCODED, HEADER, HEADER_MAP, HEADERS, HTTP_PARSE, MULTI_PART,
    PART, PATH, QUERIES, QUERY,
};
use crate::plugin::model::{HttpTargetMode, HttpVerb, ParseThreadMode, RequestMode};
use crate::plugin::parse::{
    invalid_string_map, parse_config_map_argument, parse_config_string_argument,
};
use crate::plugin::util::{config_name, label};

#[derive(Debug, Clone)]
pub(crate) struct ParsedHttpClientConfig {
    pub(crate) base_url: Option<String>,
    pub(crate) target: HttpTargetMode,
    pub(crate) parse_thread: ParseThreadMode,
    pub(crate) headers: Vec<(String, String)>,
}

pub(crate) fn parse_http_client_config(
    config: &ConfigApplicationIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> ParsedHttpClientConfig {
    let mut parsed = ParsedHttpClientConfig {
        base_url: None,
        target: HttpTargetMode::Dart,
        parse_thread: ParseThreadMode::Main,
        headers: Vec::new(),
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
                Some("dart") | Some("DustHttpTarget.dart") => parsed.target = HttpTargetMode::Dart,
                Some("flutter") | Some("DustHttpTarget.flutter") => {
                    parsed.target = HttpTargetMode::Flutter;
                }
                _ => diagnostics.push(
                    Diagnostic::error(
                        "`HttpClient(target: ...)` must be `DustHttpTarget.dart` or `DustHttpTarget.flutter`",
                    )
                    .with_label(label(config.span, "pick one of the supported target enum values")),
                ),
            },
            "parseThread" => parsed.parse_thread = parse_thread_config(config, diagnostics),
            "headers" => parsed.headers = parse_http_client_headers(config, diagnostics),
            other => diagnostics.push(
                Diagnostic::warning(format!("unknown `HttpClient` option `{other}`"))
                    .with_label(label(config.span, "remove or rename this unsupported option")),
            ),
        }
    }

    parsed
}

pub(crate) fn parse_http_client_headers(
    config: &ConfigApplicationIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<(String, String)> {
    match config.named_string_map("headers") {
        Some(values) => values,
        None => {
            diagnostics.push(invalid_string_map("HttpClient(headers: ...)", config.span));
            Vec::new()
        }
    }
}

pub(crate) fn parse_headers_config(
    config: &ConfigApplicationIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<(String, String)> {
    parse_config_map_argument(config, diagnostics, "Headers")
}

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

pub(crate) fn method_parse_thread(
    method: &MethodIr,
    default: ParseThreadMode,
    diagnostics: &mut Vec<Diagnostic>,
) -> ParseThreadMode {
    for config in &method.configs {
        if config_name(&config.symbol.0) == HTTP_PARSE {
            return parse_http_parse_config(config, diagnostics);
        }
    }
    default
}

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

pub(crate) fn method_request_mode(method: &MethodIr) -> RequestMode {
    if method
        .configs
        .iter()
        .any(|config| config_name(&config.symbol.0) == MULTI_PART)
    {
        RequestMode::MultiPart
    } else if method
        .configs
        .iter()
        .any(|config| config_name(&config.symbol.0) == FORM_URL_ENCODED)
    {
        RequestMode::FormUrlEncoded
    } else {
        RequestMode::Standard
    }
}

pub(crate) fn method_verbs(method: &MethodIr) -> Vec<HttpVerb> {
    method
        .configs
        .iter()
        .filter_map(|config| match config_name(&config.symbol.0) {
            "GET" => Some(HttpVerb::Get),
            "POST" => Some(HttpVerb::Post),
            "PUT" => Some(HttpVerb::Put),
            "PATCH" => Some(HttpVerb::Patch),
            "DELETE" => Some(HttpVerb::Delete),
            "HEAD" => Some(HttpVerb::Head),
            "OPTIONS" => Some(HttpVerb::Options),
            _ => None,
        })
        .collect()
}

pub(crate) fn method_path(method: &MethodIr, diagnostics: &mut Vec<Diagnostic>) -> Option<String> {
    for config in &method.configs {
        match config_name(&config.symbol.0) {
            "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS" => {
                return parse_config_string_argument(config, diagnostics, "HTTP verb path");
            }
            _ => {}
        }
    }
    None
}

pub(crate) fn has_config_named(configs: &[ConfigApplicationIr], expected: &str) -> bool {
    configs
        .iter()
        .any(|config| config_name(&config.symbol.0) == expected)
}

pub(crate) fn param_source_names(param: &MethodParamIr) -> Vec<&str> {
    param
        .configs
        .iter()
        .map(|config| config_name(&config.symbol.0))
        .filter(|name| {
            matches!(
                *name,
                PATH | QUERY | QUERIES | HEADER | HEADER_MAP | BODY | FIELD | PART | EXTRA
            )
        })
        .collect()
}

fn parse_thread_config(
    config: &ConfigApplicationIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> ParseThreadMode {
    let thread = config
        .named_member("parseThread")
        .or_else(|| config.named_member("thread"));
    match thread.as_deref() {
        Some("main") | Some("DustParseThread.main") => ParseThreadMode::Main,
        Some("isolate") | Some("DustParseThread.isolate") => ParseThreadMode::Isolate,
        _ => {
            diagnostics.push(
                Diagnostic::error(
                    "`parseThread` must be `DustParseThread.main` or `DustParseThread.isolate`",
                )
                .with_label(label(
                    config.span,
                    "pick one of the supported parse-thread enum values",
                )),
            );
            ParseThreadMode::Main
        }
    }
}
