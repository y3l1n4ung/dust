use crate::plugin::emit::request::{render_path_expression, render_request_data};
use crate::plugin::emit::response::render_response_return;
use crate::plugin::emit::stream::render_stream_yield;
use crate::plugin::emit::types::{is_void_type, render_fetch_type, render_type};
use crate::plugin::model::{ClientSpec, EndpointParam, EndpointSpec};

pub(crate) fn render_client_class(spec: &ClientSpec<'_>) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "final class _${} implements {} {{\n",
        spec.class_name, spec.class_name
    ));
    out.push_str(&format!(
        "  _${}(this._dio, {{String? baseUrl}}) : _baseUrl = baseUrl;\n\n",
        spec.class_name
    ));
    out.push_str("  final Dio _dio;\n");
    out.push_str("  final String? _baseUrl;\n");

    for endpoint in &spec.endpoints {
        out.push('\n');
        out.push_str(&render_endpoint_method(spec, endpoint));
    }

    out.push_str("}\n");
    out
}

fn render_endpoint_method(spec: &ClientSpec<'_>, endpoint: &EndpointSpec<'_>) -> String {
    let mut out = String::new();
    out.push_str("  @override\n");
    let async_marker = if endpoint.return_spec.is_stream() {
        "async*"
    } else {
        "async"
    };
    out.push_str(&format!(
        "  {} {}({}) {} {{\n",
        render_type(&endpoint.method.return_type),
        endpoint.method.name,
        render_method_parameters(&endpoint.method.params),
        async_marker,
    ));
    out.push_str("    final _queryParameters = <String, dynamic>{};\n");
    out.push_str("    final _headers = <String, dynamic>{};\n");
    out.push_str("    final _extra = <String, dynamic>{};\n");

    if let Some(options_param) = option_param(endpoint) {
        if options_param.ty.is_nullable() {
            out.push_str(&format!(
                "    if ({0}?.headers != null) _headers.addAll({0}!.headers!);\n",
                options_param.name
            ));
            out.push_str(&format!(
                "    if ({0}?.extra != null) _extra.addAll({0}!.extra!);\n",
                options_param.name
            ));
        } else {
            out.push_str(&format!(
                "    if ({0}.headers != null) _headers.addAll({0}.headers!);\n",
                options_param.name
            ));
            out.push_str(&format!(
                "    if ({0}.extra != null) _extra.addAll({0}.extra!);\n",
                options_param.name
            ));
        }
    }

    for (key, value) in &endpoint.headers {
        out.push_str(&format!("    _headers['{}'] = '{}';\n", key, value));
    }
    for param in &endpoint.params {
        match param {
            EndpointParam::Query { param, key } => push_map_entry(
                &mut out,
                "_queryParameters",
                &param.name,
                key,
                param.ty.is_nullable(),
            ),
            EndpointParam::Queries { param } => push_map_merge(
                &mut out,
                "_queryParameters",
                &param.name,
                param.ty.is_nullable(),
            ),
            EndpointParam::Header { param, key } => push_map_entry(
                &mut out,
                "_headers",
                &param.name,
                key,
                param.ty.is_nullable(),
            ),
            EndpointParam::HeaderMap { param } => {
                push_map_merge(&mut out, "_headers", &param.name, param.ty.is_nullable())
            }
            EndpointParam::Extra { param, key } => {
                push_map_entry(&mut out, "_extra", &param.name, key, param.ty.is_nullable())
            }
            _ => {}
        }
    }

    out.push_str(&render_request_data(endpoint));
    let content_type = match endpoint.request_mode {
        crate::plugin::model::RequestMode::Standard => "null".to_owned(),
        crate::plugin::model::RequestMode::FormUrlEncoded => {
            "'application/x-www-form-urlencoded'".to_owned()
        }
        crate::plugin::model::RequestMode::MultiPart => "'multipart/form-data'".to_owned(),
    };

    if let Some(options_name) = option_param(endpoint).map(|param| param.name.as_str()) {
        out.push_str(&format!(
            "    final _options = {0}?.copyWith(method: '{1}', headers: _headers, extra: _extra, contentType: {2}) ?? Options(method: '{1}', headers: _headers, extra: _extra, contentType: {2});\n",
            options_name,
            endpoint.verb.as_str(),
            content_type
        ));
    } else {
        out.push_str(&format!(
            "    final _options = Options(method: '{}', headers: _headers, extra: _extra, contentType: {});\n",
            endpoint.verb.as_str(),
            content_type
        ));
    }

    let base_url_expr = match &spec.base_url {
        Some(url) => format!(
            "_baseUrl ?? '{}'",
            crate::plugin::util::escape_single_quoted(url)
        ),
        None => "_baseUrl".to_owned(),
    };
    let cancel_token = param_name(endpoint, |param| {
        matches!(param, EndpointParam::CancelToken { .. })
    })
    .unwrap_or("null");
    let on_send_progress = param_name(endpoint, |param| {
        matches!(param, EndpointParam::OnSendProgress { .. })
    })
    .unwrap_or("null");
    let on_receive_progress = param_name(endpoint, |param| {
        matches!(param, EndpointParam::OnReceiveProgress { .. })
    })
    .unwrap_or("null");

    if endpoint.return_spec.is_stream() {
        out.push_str("    final _result = await _dio.fetch<ResponseBody>(\n");
    } else if is_void_type(&endpoint.return_spec.ty) && !endpoint.return_spec.raw_response {
        out.push_str(&format!(
            "    await _dio.fetch<{}>(\n",
            render_fetch_type(&endpoint.return_spec.ty)
        ));
    } else {
        out.push_str(&format!(
            "    final _result = await _dio.fetch<{}>(\n",
            render_fetch_type(&endpoint.return_spec.ty)
        ));
    }
    let stream_type = if endpoint.return_spec.is_stream() {
        "ResponseBody".to_owned()
    } else {
        render_type(&endpoint.return_spec.ty)
    };
    out.push_str(&format!("      _setStreamType<{}>(\n", stream_type));
    out.push_str("        _options\n");
    out.push_str("            .compose(\n");
    out.push_str("              _dio.options,\n");
    out.push_str(&format!(
        "              {},\n",
        render_path_expression(endpoint)
    ));
    out.push_str("              queryParameters: _queryParameters,\n");
    out.push_str("              data: _data,\n");
    out.push_str(&format!("              cancelToken: {},\n", cancel_token));
    out.push_str(&format!(
        "              onSendProgress: {},\n",
        on_send_progress
    ));
    out.push_str(&format!(
        "              onReceiveProgress: {},\n",
        on_receive_progress
    ));
    out.push_str("            )\n");
    out.push_str("            .copyWith(\n");
    out.push_str(&format!(
        "              baseUrl: _combineBaseUrls(_dio.options.baseUrl, {}),\n",
        base_url_expr
    ));
    out.push_str("            ),\n");
    out.push_str("      ),\n");
    out.push_str("    );\n");
    if endpoint.return_spec.is_stream() {
        out.push_str(&render_stream_yield(endpoint));
    } else {
        out.push_str(&render_response_return(spec, endpoint));
    }
    out.push_str("  }\n");
    out
}

fn render_method_parameters(params: &[dust_ir::MethodParamIr]) -> String {
    let positional = params
        .iter()
        .filter(|param| param.kind == dust_ir::ParamKind::Positional)
        .map(render_method_parameter)
        .collect::<Vec<_>>();
    let named = params
        .iter()
        .filter(|param| param.kind == dust_ir::ParamKind::Named)
        .map(render_method_parameter)
        .collect::<Vec<_>>();

    match (positional.is_empty(), named.is_empty()) {
        (true, true) => String::new(),
        (false, true) => positional.join(", "),
        (true, false) => format!("{{{}}}", named.join(", ")),
        (false, false) => format!("{}, {{{}}}", positional.join(", "), named.join(", ")),
    }
}

fn render_method_parameter(param: &dust_ir::MethodParamIr) -> String {
    if param.kind == dust_ir::ParamKind::Named && !param.ty.is_nullable() && !param.has_default {
        format!("required {} {}", render_type(&param.ty), param.name)
    } else {
        format!("{} {}", render_type(&param.ty), param.name)
    }
}

fn option_param<'a>(endpoint: &'a EndpointSpec<'_>) -> Option<&'a dust_ir::MethodParamIr> {
    endpoint.params.iter().find_map(|param| match param {
        EndpointParam::Options { param } => Some(*param),
        _ => None,
    })
}

fn param_name<'a, F>(endpoint: &'a EndpointSpec<'_>, matches: F) -> Option<&'a str>
where
    F: Fn(&EndpointParam<'_>) -> bool,
{
    endpoint
        .params
        .iter()
        .find(|param| matches(param))
        .map(|param| match param {
            EndpointParam::CancelToken { param }
            | EndpointParam::Options { param }
            | EndpointParam::OnSendProgress { param }
            | EndpointParam::OnReceiveProgress { param } => param.name.as_str(),
            _ => unreachable!("filtered to special params"),
        })
}

fn push_map_entry(out: &mut String, target: &str, name: &str, key: &str, nullable: bool) {
    let key = crate::plugin::util::escape_single_quoted(key);
    if nullable {
        out.push_str(&format!(
            "    if ({name} != null) {target}['{key}'] = {name};\n"
        ));
    } else {
        out.push_str(&format!("    {target}['{key}'] = {name};\n"));
    }
}

fn push_map_merge(out: &mut String, target: &str, name: &str, nullable: bool) {
    if nullable {
        out.push_str(&format!(
            "    if ({name} != null) {target}.addAll({name});\n"
        ));
    } else {
        out.push_str(&format!("    {target}.addAll({name});\n"));
    }
}
