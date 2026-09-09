//! Rendering the body of one endpoint method: its setup, its request options and the call itself.

use super::*;

/// Renders the full generated body for an endpoint method.
pub(super) fn render_endpoint_body(spec: &ClientSpec<'_>, endpoint: &EndpointSpec<'_>) -> String {
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
        include_str!("../templates/endpoint_body.jinja"),
        EndpointBodyContext {
            setup: render_endpoint_setup(endpoint),
            request_data: chunk(render_request_data(endpoint)),
            options: chunk(render_options(endpoint, &content_type)),
            fetch: chunk(render_template(
                "dio_fetch",
                include_str!("../templates/dio_fetch.jinja"),
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

/// Renders setup statements before the Dio request is issued.
pub(super) fn render_endpoint_setup(endpoint: &EndpointSpec<'_>) -> String {
    let mut setup = Vec::new();
    if let Some(options_param) = option_param(endpoint) {
        setup.push(render_template(
            if options_param.ty.is_nullable() {
                "option_param_nullable"
            } else {
                "option_param_nonnullable"
            },
            if options_param.ty.is_nullable() {
                include_str!("../templates/option_param_nullable.jinja")
            } else {
                include_str!("../templates/option_param_nonnullable.jinja")
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
            EndpointParam::Header { param, key } => setup.push(render_map_entry_with_guard(
                "_headers",
                &format!("{}.toString()", param.name),
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

/// Renders the generated Dio `Options` expression for an endpoint.
pub(super) fn render_options(endpoint: &EndpointSpec<'_>, content_type: &str) -> String {
    if let Some(options_name) = option_param(endpoint).map(|param| param.name.as_str()) {
        render_template(
            "options_with_param",
            include_str!("../templates/options_with_param.jinja"),
            OptionsContext {
                options_name,
                verb: endpoint.verb.as_str(),
                content_type,
            },
        )
    } else {
        render_template(
            "options_plain",
            include_str!("../templates/options_plain.jinja"),
            PlainOptionsContext {
                verb: endpoint.verb.as_str(),
                content_type,
            },
        )
    }
}
