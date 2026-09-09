use dust_dart_emit::render_template;
use serde::Serialize;

use crate::plugin::emit::{
    request::{render_path_expression, render_request_data},
    response::render_response_return,
    stream::render_stream_yield,
    types::{is_void_type, render_fetch_type, render_type},
};
use crate::plugin::model::{ClientSpec, EndpointParam, EndpointSpec};

/// Rendering a method's parameter list, and the map entries a request body builds.
mod params;
use self::params::*;

/// Rendering the body of one endpoint method: its setup, its request options and the call itself.
mod body;
use self::body::*;

/// Template context for the generated HTTP client class.
#[derive(Serialize)]
struct ClientClassContext<'a> {
    /// Name of the generated Dart client class.
    class_name: &'a str,
    /// Rendered endpoint method declarations.
    methods: String,
}

/// Template context for one generated endpoint method.
#[derive(Serialize)]
struct EndpointMethodContext<'a> {
    /// Rendered Dart return type.
    return_type: String,
    /// Dart method name copied from source IR.
    method_name: &'a str,
    /// Rendered Dart method parameter list, including parentheses.
    params: String,
    /// Either `async` or `async*`.
    async_marker: &'static str,
    /// Rendered method body.
    body: String,
}

/// Template context for the generated method body.
#[derive(Serialize)]
struct EndpointBodyContext {
    /// Statements that prepare headers, extras, queries, and options.
    setup: String,
    /// Rendered Dio request data block.
    request_data: String,
    /// Rendered Dio options block.
    options: String,
    /// Rendered `_dio.fetch` block.
    fetch: String,
    /// Return or stream-forwarding statements after fetch.
    completion: String,
}

/// Template context that only needs a Dart identifier.
#[derive(Serialize)]
struct NameContext<'a> {
    /// Dart identifier supplied to the template.
    name: &'a str,
}

/// Template context for assigning one keyed map entry.
#[derive(Serialize)]
struct MapEntryContext<'a> {
    /// Dart map variable receiving the entry.
    target: &'a str,
    /// Dart expression assigned to the entry.
    name: &'a str,
    /// Dart expression used by the null guard.
    guard: &'a str,
    /// Escaped string key for the map entry.
    key: String,
}

/// Template context for merging a Dart map parameter.
#[derive(Serialize)]
struct MapMergeContext<'a> {
    /// Dart map variable receiving merged entries.
    target: &'a str,
    /// Dart map parameter name to merge.
    name: &'a str,
}

/// Template context for Dio options when a user options parameter exists.
#[derive(Serialize)]
struct OptionsContext<'a> {
    /// Dart options parameter name.
    options_name: &'a str,
    /// HTTP verb string for the generated options.
    verb: &'a str,
    /// Content-type expression for the generated request.
    content_type: &'a str,
}

/// Template context for generated Dio options without a user parameter.
#[derive(Serialize)]
struct PlainOptionsContext<'a> {
    /// HTTP verb string for the generated options.
    verb: &'a str,
    /// Content-type expression for the generated request.
    content_type: &'a str,
}

/// Template context for the `_dio.fetch` call.
#[derive(Serialize)]
struct FetchContext {
    /// Assignment prefix, or bare await for void methods.
    assignment: String,
    /// Dio stream response type expression.
    stream_type: String,
    /// Rendered path expression for the request.
    path_expr: String,
    /// Cancel token expression.
    cancel_token: String,
    /// Upload progress callback expression.
    on_send_progress: String,
    /// Download progress callback expression.
    on_receive_progress: String,
    /// Base URL expression after class-level defaults are applied.
    base_url_expr: String,
}

/// Renders the generated HTTP client class for a validated spec.
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
                    .map(|endpoint| render_endpoint_method(spec, endpoint))
                    .collect::<Vec<_>>()
                    .join("\n\n"),
            },
        )
    )
}

/// Renders one generated Dart method for an endpoint.
fn render_endpoint_method(spec: &ClientSpec<'_>, endpoint: &EndpointSpec<'_>) -> String {
    let async_marker = if endpoint.return_spec.is_stream() {
        "async*"
    } else {
        "async"
    };
    let return_type = render_type(&endpoint.method.return_type);
    let params = render_method_parameters(
        &endpoint.method.params,
        &return_type,
        &endpoint.method.name,
        async_marker,
    );
    let body = render_endpoint_body(spec, endpoint);
    render_template(
        "endpoint_method",
        include_str!("templates/endpoint_method.jinja"),
        EndpointMethodContext {
            return_type,
            method_name: &endpoint.method.name,
            params,
            async_marker,
            body,
        },
    )
}
