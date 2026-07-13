use crate::ResolvedMethod;
use dust_ir::{
    ConfigApplicationIr, HttpClientConfigIr, HttpConfigIr, HttpParameterConfigIr,
    HttpParseConfigIr, HttpParseThreadIr, HttpRequestModeIr, HttpTargetIr, HttpVerbIr,
    NormalizedConfigIr,
};

/// Normalizes HTTP annotations after symbols and parser-owned values resolve.
pub(crate) fn normalize_http(
    class_configs: &mut [ConfigApplicationIr],
    methods: &mut [ResolvedMethod],
) {
    for config in class_configs {
        normalize_config(config);
    }
    for method in methods {
        for config in &mut method.configs {
            normalize_config(config);
        }
        for param in &mut method.params {
            for config in &mut param.configs {
                normalize_config(config);
            }
        }
    }
}

/// Converts one resolved HTTP annotation into typed configuration.
fn normalize_config(config: &mut ConfigApplicationIr) {
    let symbol = config.symbol.0.as_str();
    let normalized = match symbol {
        "dust_dart::HttpClient" => client_config(config),
        "dust_dart::GET" => verb_config(config, HttpVerbIr::Get),
        "dust_dart::POST" => verb_config(config, HttpVerbIr::Post),
        "dust_dart::PUT" => verb_config(config, HttpVerbIr::Put),
        "dust_dart::PATCH" => verb_config(config, HttpVerbIr::Patch),
        "dust_dart::DELETE" => verb_config(config, HttpVerbIr::Delete),
        "dust_dart::HEAD" => verb_config(config, HttpVerbIr::Head),
        "dust_dart::OPTIONS" => verb_config(config, HttpVerbIr::Options),
        "dust_dart::HttpParse" => parse_config(config),
        "dust_dart::Headers" => headers_config(config),
        "dust_dart::FormUrlEncoded" => {
            Some(HttpConfigIr::RequestMode(HttpRequestModeIr::FormUrlEncoded))
        }
        "dust_dart::MultiPart" => Some(HttpConfigIr::RequestMode(HttpRequestModeIr::MultiPart)),
        "dust_dart::Path" => Some(HttpConfigIr::Parameter(HttpParameterConfigIr::Path(
            config.positional_string(0),
        ))),
        "dust_dart::Query" => config
            .positional_string(0)
            .map(|key| HttpConfigIr::Parameter(HttpParameterConfigIr::Query(key))),
        "dust_dart::Queries" => Some(HttpConfigIr::Parameter(HttpParameterConfigIr::Queries)),
        "dust_dart::Header" => config
            .positional_string(0)
            .map(|key| HttpConfigIr::Parameter(HttpParameterConfigIr::Header(key))),
        "dust_dart::HeaderMap" => Some(HttpConfigIr::Parameter(HttpParameterConfigIr::HeaderMap)),
        "dust_dart::Body" => Some(HttpConfigIr::Parameter(HttpParameterConfigIr::Body)),
        "dust_dart::Field" => config
            .positional_string(0)
            .map(|key| HttpConfigIr::Parameter(HttpParameterConfigIr::Field(key))),
        "dust_dart::Part" => config
            .positional_string(0)
            .map(|key| HttpConfigIr::Parameter(HttpParameterConfigIr::Part(key))),
        "dust_dart::Extra" => config
            .positional_string(0)
            .map(|key| HttpConfigIr::Parameter(HttpParameterConfigIr::Extra(key))),
        _ => None,
    };
    if let Some(normalized) = normalized {
        config.normalized = Some(NormalizedConfigIr::Http(normalized));
    }
}

/// Converts an HTTP client annotation into typed client configuration.
fn client_config(config: &ConfigApplicationIr) -> Option<HttpConfigIr> {
    let target = match config.named_member("target").as_deref() {
        None | Some("dart") | Some("HttpTarget.dart") => HttpTargetIr::Dart,
        Some("flutter") | Some("HttpTarget.flutter") => HttpTargetIr::Flutter,
        Some(_) => return None,
    };
    let parse_thread = match config.named_member("parseThread").as_deref() {
        None | Some("main") | Some("HttpParseThread.main") => HttpParseThreadIr::Main,
        Some("isolate") | Some("HttpParseThread.isolate") => HttpParseThreadIr::Isolate,
        Some(_) => return None,
    };
    let headers = config.named_string_map("headers").unwrap_or_default();
    let generate_test = config.named_bool("generateTest").unwrap_or(false);
    Some(HttpConfigIr::Client(HttpClientConfigIr {
        base_url: config.named_string("baseUrl"),
        target,
        parse_thread,
        headers,
        generate_test,
    }))
}

/// Converts one HTTP verb annotation into typed verb configuration.
fn verb_config(config: &ConfigApplicationIr, verb: HttpVerbIr) -> Option<HttpConfigIr> {
    let path = config.positional_string(0)?;
    Some(HttpConfigIr::Verb { verb, path })
}

/// Converts an HTTP parse annotation into typed parse configuration.
fn parse_config(config: &ConfigApplicationIr) -> Option<HttpConfigIr> {
    let thread = match config.named_member("thread").as_deref() {
        None | Some("main") | Some("HttpParseThread.main") => HttpParseThreadIr::Main,
        Some("isolate") | Some("HttpParseThread.isolate") => HttpParseThreadIr::Isolate,
        Some(_) => return None,
    };
    Some(HttpConfigIr::Parse(HttpParseConfigIr { thread }))
}

/// Converts an HTTP headers annotation into typed header configuration.
fn headers_config(config: &ConfigApplicationIr) -> Option<HttpConfigIr> {
    let values = config.positional_string_map(0)?;
    Some(HttpConfigIr::Headers(values))
}
