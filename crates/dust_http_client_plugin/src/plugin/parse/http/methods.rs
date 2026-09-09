//! Reading the verb, path, request mode and parameter sources off one method.

use super::*;

/// Resolves the request encoding mode for a method.
pub(crate) fn method_request_mode(method: &MethodIr) -> RequestMode {
    for config in &method.configs {
        if let Some(NormalizedConfigIr::Http(HttpConfigIr::RequestMode(mode))) =
            config.normalized.as_ref()
        {
            return match mode {
                dust_ir::HttpRequestModeIr::Standard => RequestMode::Standard,
                dust_ir::HttpRequestModeIr::FormUrlEncoded => RequestMode::FormUrlEncoded,
                dust_ir::HttpRequestModeIr::MultiPart => RequestMode::MultiPart,
            };
        }
    }
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

/// Returns all HTTP verb annotations attached to a method.
pub(crate) fn method_verbs(method: &MethodIr) -> Vec<HttpVerb> {
    let normalized = method
        .configs
        .iter()
        .filter_map(|config| match config.normalized.as_ref() {
            Some(NormalizedConfigIr::Http(HttpConfigIr::Verb { verb, .. })) => Some(match verb {
                dust_ir::HttpVerbIr::Get => HttpVerb::Get,
                dust_ir::HttpVerbIr::Post => HttpVerb::Post,
                dust_ir::HttpVerbIr::Put => HttpVerb::Put,
                dust_ir::HttpVerbIr::Patch => HttpVerb::Patch,
                dust_ir::HttpVerbIr::Delete => HttpVerb::Delete,
                dust_ir::HttpVerbIr::Head => HttpVerb::Head,
                dust_ir::HttpVerbIr::Options => HttpVerb::Options,
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    if !normalized.is_empty() {
        return normalized;
    }
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

/// Parses the path string from a method's HTTP verb annotation.
pub(crate) fn method_path(method: &MethodIr, diagnostics: &mut Vec<Diagnostic>) -> Option<String> {
    for config in &method.configs {
        match config_name(&config.symbol.0) {
            "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS" => {
                if let Some(NormalizedConfigIr::Http(HttpConfigIr::Verb { path, .. })) =
                    config.normalized.as_ref()
                {
                    return Some(path.clone());
                }
                return parse_config_string_argument(config, diagnostics, "HTTP verb path");
            }
            _ => {}
        }
    }
    None
}

/// Returns true when a config list contains the requested short name.
pub(crate) fn has_config_named(configs: &[ConfigApplicationIr], expected: &str) -> bool {
    configs
        .iter()
        .any(|config| config_name(&config.symbol.0) == expected)
}

/// Returns HTTP source annotations attached to a parameter.
pub(crate) fn param_source_names(param: &MethodParamIr) -> Vec<&str> {
    let normalized = param
        .configs
        .iter()
        .filter_map(|config| match config.normalized.as_ref() {
            Some(NormalizedConfigIr::Http(HttpConfigIr::Parameter(binding))) => {
                Some(match binding {
                    HttpParameterConfigIr::Path(_) => PATH,
                    HttpParameterConfigIr::Query(_) => QUERY,
                    HttpParameterConfigIr::Queries => QUERIES,
                    HttpParameterConfigIr::Header(_) => HEADER,
                    HttpParameterConfigIr::HeaderMap => HEADER_MAP,
                    HttpParameterConfigIr::Body => BODY,
                    HttpParameterConfigIr::Field(_) => FIELD,
                    HttpParameterConfigIr::Part(_) => PART,
                    HttpParameterConfigIr::Extra(_) => EXTRA,
                })
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    if !normalized.is_empty() {
        return normalized;
    }
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
