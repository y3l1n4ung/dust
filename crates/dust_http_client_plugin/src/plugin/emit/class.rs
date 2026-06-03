use dust_dart_emit::render_template;
use serde::Serialize;

use crate::plugin::emit::{
    request::{render_path_expression, render_request_data},
    response::render_response_return,
    stream::render_stream_yield,
    types::{is_void_type, render_fetch_type, render_type},
};
use crate::plugin::model::{ClientSpec, EndpointParam, EndpointSpec};

#[derive(Serialize)]
struct ClientClassContext<'a> {
    class_name: &'a str,
    methods: String,
}

#[derive(Serialize)]
struct EndpointMethodContext<'a> {
    return_type: String,
    method_name: &'a str,
    params: String,
    async_marker: &'static str,
    body: String,
}

#[derive(Serialize)]
struct EndpointBodyContext {
    setup: String,
    request_data: String,
    options: String,
    fetch: String,
    completion: String,
}

#[derive(Serialize)]
struct NameContext<'a> {
    name: &'a str,
}

#[derive(Serialize)]
struct MapEntryContext<'a> {
    target: &'a str,
    name: &'a str,
    key: String,
}

#[derive(Serialize)]
struct MapMergeContext<'a> {
    target: &'a str,
    name: &'a str,
}

#[derive(Serialize)]
struct OptionsContext<'a> {
    options_name: &'a str,
    verb: &'a str,
    content_type: &'a str,
}

#[derive(Serialize)]
struct PlainOptionsContext<'a> {
    verb: &'a str,
    content_type: &'a str,
}

#[derive(Serialize)]
struct FetchContext {
    assignment: String,
    stream_type: String,
    path_expr: String,
    cancel_token: String,
    on_send_progress: String,
    on_receive_progress: String,
    base_url_expr: String,
}

pub(crate) fn render_client_class(spec: &ClientSpec<'_>) -> String {
    format!(
        "{}\n",
        render_template(
            "client_class",
            include_str!("templates/client_class.jinja"),
            ClientClassContext {
                class_name: spec.class_name,
                methods: spec
                    .endpoints
                    .iter()
                    .map(|endpoint| format!("\n{}", render_endpoint_method(spec, endpoint)))
                    .collect::<Vec<_>>()
                    .join(""),
            },
        )
    )
}

fn render_endpoint_method(spec: &ClientSpec<'_>, endpoint: &EndpointSpec<'_>) -> String {
    let async_marker = if endpoint.return_spec.is_stream() {
        "async*"
    } else {
        "async"
    };
    let body = render_endpoint_body(spec, endpoint);
    render_template(
        "endpoint_method",
        include_str!("templates/endpoint_method.jinja"),
        EndpointMethodContext {
            return_type: render_type(&endpoint.method.return_type),
            method_name: &endpoint.method.name,
            params: render_method_parameters(&endpoint.method.params),
            async_marker,
            body,
        },
    )
}

fn render_endpoint_body(spec: &ClientSpec<'_>, endpoint: &EndpointSpec<'_>) -> String {
    let content_type = match endpoint.request_mode {
        crate::plugin::model::RequestMode::Standard => "null".to_owned(),
        crate::plugin::model::RequestMode::FormUrlEncoded => {
            "'application/x-www-form-urlencoded'".to_owned()
        }
        crate::plugin::model::RequestMode::MultiPart => "'multipart/form-data'".to_owned(),
    };

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

    let stream_type = if endpoint.return_spec.is_stream() {
        "ResponseBody".to_owned()
    } else {
        render_type(&endpoint.return_spec.ty)
    };

    let assignment = if endpoint.return_spec.is_stream() {
        "    final _result = await _dio.fetch<ResponseBody>(\n".to_owned()
    } else if is_void_type(&endpoint.return_spec.ty) && !endpoint.return_spec.raw_response {
        format!(
            "    await _dio.fetch<{}>(\n",
            render_fetch_type(&endpoint.return_spec.ty)
        )
    } else {
        format!(
            "    final _result = await _dio.fetch<{}>(\n",
            render_fetch_type(&endpoint.return_spec.ty)
        )
    };

    render_template(
        "endpoint_body",
        include_str!("templates/endpoint_body.jinja"),
        EndpointBodyContext {
            setup: render_endpoint_setup(endpoint),
            request_data: chunk(render_request_data(endpoint)),
            options: chunk(render_options(endpoint, &content_type)),
            fetch: chunk(render_template(
                "dio_fetch",
                include_str!("templates/dio_fetch.jinja"),
                FetchContext {
                    assignment,
                    stream_type,
                    path_expr: render_path_expression(endpoint),
                    cancel_token: cancel_token.to_owned(),
                    on_send_progress: on_send_progress.to_owned(),
                    on_receive_progress: on_receive_progress.to_owned(),
                    base_url_expr,
                },
            )),
            completion: if endpoint.return_spec.is_stream() {
                chunk(render_stream_yield(endpoint))
            } else {
                chunk(render_response_return(spec, endpoint))
            },
        },
    )
}

fn render_endpoint_setup(endpoint: &EndpointSpec<'_>) -> String {
    let mut setup = Vec::new();
    if let Some(options_param) = option_param(endpoint) {
        setup.push(render_template(
            if options_param.ty.is_nullable() {
                "option_param_nullable"
            } else {
                "option_param_nonnullable"
            },
            if options_param.ty.is_nullable() {
                include_str!("templates/option_param_nullable.jinja")
            } else {
                include_str!("templates/option_param_nonnullable.jinja")
            },
            NameContext {
                name: &options_param.name,
            },
        ));
    }

    for (key, value) in &endpoint.headers {
        setup.push(render_map_entry(
            "_headers",
            &format!("'{}'", crate::plugin::util::escape_single_quoted(value)),
            key,
            false,
        ));
    }
    for param in &endpoint.params {
        match param {
            EndpointParam::Query { param, key } => setup.push(render_map_entry(
                "_queryParameters",
                &param.name,
                key,
                param.ty.is_nullable(),
            )),
            EndpointParam::Queries { param } => setup.push(render_map_merge(
                "_queryParameters",
                &param.name,
                param.ty.is_nullable(),
            )),
            EndpointParam::Header { param, key } => setup.push(render_map_entry(
                "_headers",
                &param.name,
                key,
                param.ty.is_nullable(),
            )),
            EndpointParam::HeaderMap { param } => setup.push(render_map_merge(
                "_headers",
                &param.name,
                param.ty.is_nullable(),
            )),
            EndpointParam::Extra { param, key } => setup.push(render_map_entry(
                "_extra",
                &param.name,
                key,
                param.ty.is_nullable(),
            )),
            _ => {}
        }
    }
    join_chunks(setup)
}

fn render_options(endpoint: &EndpointSpec<'_>, content_type: &str) -> String {
    if let Some(options_name) = option_param(endpoint).map(|param| param.name.as_str()) {
        render_template(
            "options_with_param",
            include_str!("templates/options_with_param.jinja"),
            OptionsContext {
                options_name,
                verb: endpoint.verb.as_str(),
                content_type,
            },
        )
    } else {
        render_template(
            "options_plain",
            include_str!("templates/options_plain.jinja"),
            PlainOptionsContext {
                verb: endpoint.verb.as_str(),
                content_type,
            },
        )
    }
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

fn render_map_entry(target: &str, name: &str, key: &str, nullable: bool) -> String {
    if nullable {
        render_template(
            "map_entry_nullable",
            include_str!("templates/map_entry_nullable.jinja"),
            MapEntryContext {
                target,
                name,
                key: crate::plugin::util::escape_single_quoted(key),
            },
        )
    } else {
        render_template(
            "map_entry",
            include_str!("templates/map_entry.jinja"),
            MapEntryContext {
                target,
                name,
                key: crate::plugin::util::escape_single_quoted(key),
            },
        )
    }
}

fn render_map_merge(target: &str, name: &str, nullable: bool) -> String {
    if nullable {
        render_template(
            "map_merge_nullable",
            include_str!("templates/map_merge_nullable.jinja"),
            MapMergeContext { target, name },
        )
    } else {
        render_template(
            "map_merge",
            include_str!("templates/map_merge.jinja"),
            MapMergeContext { target, name },
        )
    }
}

fn chunk(mut value: String) -> String {
    if !value.is_empty() && !value.ends_with('\n') {
        value.push('\n');
    }
    value
}

fn join_chunks(chunks: Vec<String>) -> String {
    chunks.into_iter().map(chunk).collect()
}
