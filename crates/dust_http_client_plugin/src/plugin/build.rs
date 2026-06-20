use dust_dart_emit::{DART_FUTURE, DART_LIST, DART_MAP, DART_RESPONSE, DART_STREAM, DART_VOID};
use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, MethodParamIr, TypeIr};

use crate::plugin::constants::{
    BODY, EXTRA, FIELD, HEADER, HEADER_MAP, HTTP_CLIENT, PART, PATH, QUERIES, QUERY,
};
use crate::plugin::model::{ClientSpec, EndpointParam, EndpointSpec, ReturnMode, ReturnSpec};
use crate::plugin::parse::{
    ParsedHttpClientConfig, has_config_named, method_parse_thread, method_path,
    method_request_mode, method_verbs, param_source_names, parse_http_client_config,
    parse_method_headers, parse_optional_string_argument, parse_required_string_argument,
};
use crate::plugin::util::{
    is_list_of_int_type, is_response_body_type, is_string_keyed_map, is_string_type, type_name_is,
};
use crate::plugin::validate::validate_client_class;

/// Special Dio parameter role inferred from the Dart parameter type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EndpointParamKind {
    /// Dio `CancelToken` passed through to `_dio.fetch`.
    CancelToken,
    /// Dio `Options` merged into generated request options.
    Options,
    /// Dio upload progress callback.
    OnSendProgress,
    /// Dio download progress callback.
    OnReceiveProgress,
    /// A `ProgressCallback` parameter whose source name is not supported.
    UnsupportedProgressName,
}

/// Builds a validated client spec for an annotated class.
pub(super) fn build_client_spec<'a>(
    imports: &[String],
    class: &'a ClassIr,
) -> Result<ClientSpec<'a>, Vec<Diagnostic>> {
    if !has_config_named(&class.configs, HTTP_CLIENT) {
        return Err(Vec::new());
    }

    let mut diagnostics = validate_client_class(imports, class);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let Some(http_client) = class
        .configs
        .iter()
        .find(|config| crate::plugin::util::config_name(&config.symbol.0) == HTTP_CLIENT)
    else {
        return Err(diagnostics);
    };
    let ParsedHttpClientConfig {
        base_url,
        target: _,
        parse_thread,
        headers,
    } = parse_http_client_config(http_client, &mut diagnostics);

    let endpoints = class
        .methods
        .iter()
        .filter_map(|method| {
            build_endpoint_spec(class, method, parse_thread, &headers, &mut diagnostics)
        })
        .collect::<Vec<_>>();

    if diagnostics.is_empty() {
        Ok(ClientSpec {
            class_name: &class.name,
            base_url,
            endpoints,
        })
    } else {
        Err(diagnostics)
    }
}

/// Builds a validated endpoint spec for a method with an HTTP verb annotation.
pub(super) fn build_endpoint_spec<'a>(
    _class: &'a ClassIr,
    method: &'a dust_ir::MethodIr,
    default_parse_thread: crate::plugin::model::ParseThreadMode,
    class_headers: &[(String, String)],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<EndpointSpec<'a>> {
    let verb = method_verbs(method).first().copied()?;
    let path = method_path(method, diagnostics)?;
    let parse_thread = method_parse_thread(method, default_parse_thread, diagnostics);
    let request_mode = method_request_mode(method);
    let return_spec = classify_return_type(&method.return_type)?;

    let mut params = Vec::new();
    for param in &method.params {
        let names = param_source_names(param);
        let endpoint_param = if names.is_empty() {
            match special_param_kind(param)? {
                EndpointParamKind::CancelToken => EndpointParam::CancelToken { param },
                EndpointParamKind::Options => EndpointParam::Options { param },
                EndpointParamKind::OnSendProgress => EndpointParam::OnSendProgress { param },
                EndpointParamKind::OnReceiveProgress => EndpointParam::OnReceiveProgress { param },
                EndpointParamKind::UnsupportedProgressName => return None,
            }
        } else {
            match names[0] {
                PATH => EndpointParam::Path {
                    param,
                    key: parse_optional_string_argument(param, PATH, diagnostics)
                        .unwrap_or_else(|| param.name.clone()),
                },
                QUERY => EndpointParam::Query {
                    param,
                    key: parse_required_string_argument(param, QUERY, diagnostics)?,
                },
                QUERIES => EndpointParam::Queries { param },
                HEADER => EndpointParam::Header {
                    param,
                    key: parse_required_string_argument(param, HEADER, diagnostics)?,
                },
                HEADER_MAP => EndpointParam::HeaderMap { param },
                BODY => EndpointParam::Body { param },
                FIELD => EndpointParam::Field {
                    param,
                    key: parse_required_string_argument(param, FIELD, diagnostics)?,
                },
                PART => EndpointParam::Part {
                    param,
                    key: parse_required_string_argument(param, PART, diagnostics)?,
                },
                EXTRA => EndpointParam::Extra {
                    param,
                    key: parse_required_string_argument(param, EXTRA, diagnostics)?,
                },
                _ => return None,
            }
        };
        params.push(endpoint_param);
    }

    let mut headers = class_headers.to_vec();
    headers.extend(parse_method_headers(method, diagnostics));

    Some(EndpointSpec {
        method,
        verb,
        path,
        parse_thread,
        headers,
        request_mode,
        params,
        return_spec,
    })
}

/// Classifies unannotated parameters that Dio recognizes by type.
pub(super) fn special_param_kind(param: &MethodParamIr) -> Option<EndpointParamKind> {
    if type_name_is(&param.ty, "CancelToken") {
        return Some(EndpointParamKind::CancelToken);
    }
    if type_name_is(&param.ty, "Options") {
        return Some(EndpointParamKind::Options);
    }
    if type_name_is(&param.ty, "ProgressCallback") {
        return match param.name.as_str() {
            "onSendProgress" => Some(EndpointParamKind::OnSendProgress),
            "onReceiveProgress" => Some(EndpointParamKind::OnReceiveProgress),
            _ => Some(EndpointParamKind::UnsupportedProgressName),
        };
    }
    None
}

/// Classifies the supported generated return shape for an endpoint method.
pub(super) fn classify_return_type(return_type: &TypeIr) -> Option<ReturnSpec> {
    if type_name_is(return_type, DART_FUTURE) && return_type.args().len() == 1 {
        let inner = &return_type.args()[0];
        if type_name_is(inner, DART_RESPONSE) && inner.args().len() == 1 {
            let payload = &inner.args()[0];
            return is_supported_payload(payload, true).then(|| ReturnSpec {
                mode: ReturnMode::Unary,
                raw_response: true,
                ty: payload.clone(),
            });
        }
        if is_supported_payload(inner, true) {
            return Some(ReturnSpec {
                mode: ReturnMode::Unary,
                raw_response: false,
                ty: inner.clone(),
            });
        }
    }
    if type_name_is(return_type, DART_STREAM) && return_type.args().len() == 1 {
        let inner = &return_type.args()[0];
        if let Some(mode) = classify_stream_mode(inner) {
            return Some(ReturnSpec {
                mode,
                raw_response: false,
                ty: inner.clone(),
            });
        }
    }
    None
}

/// Returns true when the payload can be fetched and decoded by generated code.
fn is_supported_payload(ty: &TypeIr, allow_response_body: bool) -> bool {
    match ty {
        TypeIr::Dynamic => true,
        TypeIr::Builtin { .. } => true,
        TypeIr::Named { name, args, .. } if name.as_ref() == DART_VOID => args.is_empty(),
        TypeIr::Named { .. } if type_name_is(ty, DART_MAP) => is_string_keyed_map(ty),
        TypeIr::Named { name, args, .. } if name.rsplit('.').next() == Some(DART_LIST) => {
            args.len() == 1 && is_supported_payload(&args[0], false)
        }
        TypeIr::Named { .. } if is_response_body_type(ty) => allow_response_body,
        TypeIr::Named { .. }
            if type_name_is(ty, DART_RESPONSE) || type_name_is(ty, DART_STREAM) =>
        {
            false
        }
        TypeIr::Named { .. } => true,
        TypeIr::Function { .. } | TypeIr::Record { .. } | TypeIr::Unknown => false,
    }
}

/// Classifies `Stream<T>` payloads that Dust can render as response streams.
fn classify_stream_mode(ty: &TypeIr) -> Option<ReturnMode> {
    if is_list_of_int_type(ty) {
        return Some(ReturnMode::ByteStream);
    }
    if is_string_type(ty) {
        return Some(ReturnMode::TextStream);
    }
    None
}
