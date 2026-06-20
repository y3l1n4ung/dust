/// Parses common annotation argument shapes.
mod args;
/// Parses HTTP client annotations into plugin settings.
mod http;

pub(super) use args::{
    invalid_string_map, parse_config_map_argument, parse_config_string_argument,
    parse_optional_string_argument, parse_required_string_argument,
};
pub(super) use http::{
    ParsedHttpClientConfig, has_config_named, method_parse_thread, method_path,
    method_request_mode, method_verbs, param_source_names, parse_headers_config,
    parse_http_client_config, parse_http_client_headers, parse_http_parse_config,
    parse_method_headers,
};
